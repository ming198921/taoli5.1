import { chromium } from 'playwright';

async function testRefreshFix() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔄 测试页面刷新白屏修复...');
    
    // 测试各个路由的刷新
    const routesToTest = [
      '/dashboard',
      '/system', 
      '/logging',
      '/cleaning',
      '/strategy',
      '/performance',
      '/trading',
      '/ai-model',
      '/config'
    ];

    for (const route of routesToTest) {
      console.log(`\n🧪 测试路由: ${route}`);
      
      try {
        // 直接访问路由（模拟刷新）
        await page.goto(`http://57.183.21.242:3003${route}`, { 
          waitUntil: 'domcontentloaded',
          timeout: 10000 
        });
        
        await page.waitForTimeout(2000);
        
        // 检查页面内容
        const pageState = await page.evaluate(() => {
          const hasContent = document.body?.textContent?.trim().length > 0;
          const hasReactApp = document.querySelector('#root')?.children?.length > 0;
          const title = document.title;
          const visibleElements = document.querySelectorAll('*:not(script):not(style)').length;
          
          return {
            hasContent,
            hasReactApp,
            title,
            visibleElements,
            isWhiteScreen: !hasContent || visibleElements < 10
          };
        });
        
        console.log(`  ${route}: ${pageState.isWhiteScreen ? '❌ 白屏' : '✅ 正常'}`);
        console.log(`    - 有内容: ${pageState.hasContent}`);
        console.log(`    - React应用: ${pageState.hasReactApp}`);
        console.log(`    - 可见元素: ${pageState.visibleElements}`);
        
        // 如果是白屏，截图记录
        if (pageState.isWhiteScreen) {
          await page.screenshot({ 
            path: `/home/ubuntu/arbitrage-frontend-v5.1/refresh-issue-${route.replace('/', '')}.png`,
            fullPage: true 
          });
        }
        
      } catch (error) {
        console.log(`  ${route}: ❌ 加载失败 - ${error.message}`);
      }
    }
    
    // 测试导航功能
    console.log('\n🧪 测试导航功能...');
    await page.goto('http://57.183.21.242:3003/dashboard', { 
      waitUntil: 'domcontentloaded',
      timeout: 10000 
    });
    
    await page.waitForTimeout(2000);
    
    // 点击系统控制菜单
    const systemMenu = await page.$('li[data-menu-id*="system"]');
    if (systemMenu) {
      await systemMenu.click();
      await page.waitForTimeout(1000);
      
      const currentUrl = page.url();
      console.log(`导航到系统控制: ${currentUrl.includes('/system') ? '✅ 成功' : '❌ 失败'}`);
    }
    
    // 最终测试截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/refresh-fix-final.png',
      fullPage: true 
    });
    
    console.log('\n🎯 刷新问题修复测试完成！');

  } catch (error) {
    console.error('❌ 测试过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

testRefreshFix().catch(console.error);