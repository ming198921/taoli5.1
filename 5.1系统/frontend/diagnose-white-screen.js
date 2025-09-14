import { chromium } from 'playwright';

async function diagnoseWhiteScreen() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 诊断页面白屏问题...');
    
    // 监听控制台错误
    const consoleErrors = [];
    page.on('console', msg => {
      if (msg.type() === 'error') {
        consoleErrors.push(msg.text());
      }
    });
    
    // 监听页面错误
    const pageErrors = [];
    page.on('pageerror', error => {
      pageErrors.push(error.message);
    });
    
    // 监听网络请求失败
    const failedRequests = [];
    page.on('requestfailed', request => {
      failedRequests.push({
        url: request.url(),
        error: request.failure()?.errorText
      });
    });
    
    // 尝试访问页面
    try {
      await page.goto('http://57.183.21.242:3003/system', { 
        waitUntil: 'domcontentloaded',
        timeout: 30000 
      });
    } catch (error) {
      console.error('页面加载错误:', error.message);
    }
    
    await page.waitForTimeout(3000);
    
    // 检查页面内容
    const pageContent = await page.evaluate(() => {
      return {
        title: document.title,
        bodyText: document.body?.textContent?.trim().slice(0, 200),
        hasReactRoot: !!document.querySelector('#root'),
        rootContent: document.querySelector('#root')?.innerHTML?.slice(0, 200),
        hasAnyContent: document.body?.children?.length > 0,
        visibleElements: document.querySelectorAll('*:not(script):not(style)').length
      };
    });
    
    // 检查React错误
    const reactErrors = await page.evaluate(() => {
      const errorElement = document.querySelector('.error-boundary') || 
                          document.querySelector('[class*="error"]') ||
                          document.querySelector('#root')?.textContent?.includes('Error');
      return errorElement ? true : false;
    });
    
    // 截图当前状态
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/white-screen-diagnosis.png',
      fullPage: true 
    });
    
    console.log('\n📊 诊断结果:');
    console.log('页面内容:', pageContent);
    console.log('\n控制台错误数量:', consoleErrors.length);
    if (consoleErrors.length > 0) {
      console.log('控制台错误:', consoleErrors);
    }
    
    console.log('\n页面错误数量:', pageErrors.length);
    if (pageErrors.length > 0) {
      console.log('页面错误:', pageErrors);
    }
    
    console.log('\n失败请求数量:', failedRequests.length);
    if (failedRequests.length > 0) {
      console.log('失败请求:', failedRequests);
    }
    
    console.log('\nReact错误:', reactErrors ? '是' : '否');
    
    // 检查具体路由
    const routes = ['/system', '/dashboard', '/log', '/cleaning', '/strategy'];
    console.log('\n检查其他路由:');
    
    for (const route of routes) {
      try {
        await page.goto(`http://57.183.21.242:3003${route}`, { 
          waitUntil: 'domcontentloaded',
          timeout: 10000 
        });
        
        const hasContent = await page.evaluate(() => {
          return document.body?.textContent?.trim().length > 0;
        });
        
        console.log(`${route}: ${hasContent ? '✅ 有内容' : '❌ 白屏'}`);
      } catch (error) {
        console.log(`${route}: ❌ 加载失败 - ${error.message}`);
      }
    }
    
  } catch (error) {
    console.error('诊断过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

diagnoseWhiteScreen().catch(console.error);