import { chromium } from 'playwright';

async function checkStrategyPage() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 检查当前策略页面状态...');

    await page.goto('http://57.183.21.242:3003/strategy', { 
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
        hasStrategies: document.body.textContent.includes('策略') || document.body.textContent.includes('strategy'),
        hasMonitoring: document.body.textContent.includes('监控') || document.body.textContent.includes('monitoring'),
        hasDebug: document.body.textContent.includes('调试') || document.body.textContent.includes('debug'),
        hasHotReload: document.body.textContent.includes('热重载') || document.body.textContent.includes('热更新'),
        elementCount: document.querySelectorAll('*').length,
        bodyPreview: document.body.textContent.slice(0, 300)
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

    console.log('\n🔍 功能检查:');
    console.log(`策略管理: ${pageAnalysis.hasStrategies ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`实时监控: ${pageAnalysis.hasMonitoring ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`调试工具: ${pageAnalysis.hasDebug ? '✅ 存在' : '❌ 缺失'}`);
    console.log(`热重载: ${pageAnalysis.hasHotReload ? '✅ 存在' : '❌ 缺失'}`);

    console.log('\n📄 页面内容预览:');
    console.log(`"${pageAnalysis.bodyPreview}"`);

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/current-strategy-page.png',
      fullPage: true 
    });

    console.log('\n🎯 结论:');
    if (pageAnalysis.elementCount < 50 || !pageAnalysis.hasStrategies) {
      console.log('❌ 策略页面功能不完整，需要完全重写');
    } else {
      console.log('⚠️ 策略页面有基础结构，但可能缺少38个API接口功能');
    }

  } catch (error) {
    console.error('❌ 检查过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

checkStrategyPage().catch(console.error);