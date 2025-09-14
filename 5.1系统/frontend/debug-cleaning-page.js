import { chromium } from 'playwright';

async function debugCleaningPage() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 调试清洗页面加载问题...');
    
    // 监听控制台消息
    page.on('console', msg => {
      console.log(`Console: ${msg.text()}`);
    });
    
    page.on('pageerror', error => {
      console.log(`Page Error: ${error.message}`);
    });

    await page.goto('http://57.183.21.242:3003/cleaning', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(8000); // 等待更长时间

    // 检查页面基本结构
    const pageStructure = await page.evaluate(() => {
      return {
        title: document.querySelector('h1')?.textContent,
        hasTabsComponent: document.querySelector('.ant-tabs') !== null,
        tabCount: document.querySelectorAll('.ant-tabs-tab').length,
        hasCards: document.querySelectorAll('.ant-card').length,
        hasStats: document.querySelectorAll('.ant-statistic').length,
        bodyText: document.body.textContent.slice(0, 200)
      };
    });

    console.log('\n页面结构检查:');
    console.log(`标题: ${pageStructure.title}`);
    console.log(`Tabs组件: ${pageStructure.hasTabsComponent ? '存在' : '不存在'}`);
    console.log(`Tab数量: ${pageStructure.tabCount}`);
    console.log(`卡片数量: ${pageStructure.hasCards}`);
    console.log(`统计组件数量: ${pageStructure.hasStats}`);
    console.log(`页面内容预览: ${pageStructure.bodyText}`);

    // 检查数据加载状态
    const dataState = await page.evaluate(() => {
      const loadingElements = document.querySelectorAll('.ant-spin-spinning');
      const errorElements = document.querySelectorAll('.ant-alert-error');
      return {
        isLoading: loadingElements.length > 0,
        hasErrors: errorElements.length > 0,
        loadingCount: loadingElements.length,
        errorCount: errorElements.length
      };
    });

    console.log('\n数据状态检查:');
    console.log(`正在加载: ${dataState.isLoading ? '是' : '否'} (${dataState.loadingCount}个)`);
    console.log(`有错误: ${dataState.hasErrors ? '是' : '否'} (${dataState.errorCount}个)`);

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/cleaning-debug.png',
      fullPage: true 
    });

    console.log('\n调试完成，截图已保存');

  } catch (error) {
    console.error('调试过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

debugCleaningPage().catch(console.error);