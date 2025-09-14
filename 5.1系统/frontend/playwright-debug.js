import { chromium } from 'playwright';

async function debugDashboard() {
  const browser = await chromium.launch({ 
    headless: true, // 服务器环境使用headless模式
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    // 监听控制台消息
    page.on('console', msg => {
      const type = msg.type();
      const text = msg.text();
      if (type === 'error') {
        console.log(`❌ Console Error: ${text}`);
      } else if (type === 'warn') {
        console.log(`⚠️  Console Warning: ${text}`);
      } else if (text.includes('API') || text.includes('service') || text.includes('健康')) {
        console.log(`📋 Console Log: ${text}`);
      }
    });

    // 监听网络请求
    page.on('response', response => {
      if (response.url().includes('/api/') || response.url().includes(':400')) {
        const status = response.status();
        const url = response.url();
        if (status >= 400) {
          console.log(`❌ API Failed: ${status} ${url}`);
        } else {
          console.log(`✅ API Success: ${status} ${url}`);
        }
      }
    });

    // 监听页面错误
    page.on('pageerror', error => {
      console.log(`💥 Page Error: ${error.message}`);
    });

    console.log('🚀 正在访问Dashboard页面...');
    
    // 访问页面
    await page.goto('http://57.183.21.242:3003/dashboard', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    console.log('📸 正在截图分析页面状态...');
    
    // 截图1：初始加载状态
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/screenshot-initial.png',
      fullPage: true 
    });

    // 等待页面组件加载
    console.log('⏰ 等待组件加载...');
    await page.waitForTimeout(5000);

    // 截图2：组件加载后状态  
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/screenshot-loaded.png',
      fullPage: true 
    });

    // 分析页面组件加载状态
    const pageAnalysis = await page.evaluate(() => {
      const analysis = {};
      
      // 检查统计卡片
      const statCards = document.querySelectorAll('.ant-statistic, [class*="statistic"]');
      analysis.statCards = Array.from(statCards).map(card => ({
        title: card.querySelector('.ant-statistic-title')?.textContent || 'Unknown',
        value: card.querySelector('.ant-statistic-content-value')?.textContent || 'No Value',
        visible: card.offsetParent !== null
      }));
      
      // 检查服务状态表格/卡片
      const serviceCards = document.querySelectorAll('.ant-card');
      analysis.serviceCards = serviceCards.length;
      
      // 检查加载状态
      const loadingElements = document.querySelectorAll('.ant-spin, [class*="loading"]');
      analysis.isLoading = loadingElements.length > 0;
      
      // 检查错误信息
      const errorElements = document.querySelectorAll('.ant-alert-error, [class*="error"]');
      analysis.errors = Array.from(errorElements).map(el => el.textContent);
      
      // 检查API数据显示
      const apiData = document.querySelector('[data-testid*="api"], [class*="api"]');
      analysis.hasApiData = !!apiData;
      
      return analysis;
    });

    console.log('\n📊 页面组件分析结果:');
    console.log(`- 统计卡片数量: ${pageAnalysis.statCards.length}`);
    pageAnalysis.statCards.forEach(card => {
      console.log(`  - ${card.title}: ${card.value} (${card.visible ? '显示' : '隐藏'})`);
    });
    
    console.log(`- 服务卡片数量: ${pageAnalysis.serviceCards}`);
    console.log(`- 页面是否在加载: ${pageAnalysis.isLoading}`);
    console.log(`- 错误信息数量: ${pageAnalysis.errors.length}`);
    
    if (pageAnalysis.errors.length > 0) {
      console.log('❌ 发现页面错误:');
      pageAnalysis.errors.forEach(error => console.log(`  - ${error}`));
    }

    // 服务器环境跳过开发者工具

    // 检查网络请求详情
    const networkEntries = await page.evaluate(() => {
      return performance.getEntries()
        .filter(entry => entry.name.includes('/api/') || entry.name.includes('health'))
        .map(entry => ({
          url: entry.name,
          duration: Math.round(entry.duration),
          responseStart: entry.responseStart,
          transferSize: entry.transferSize || 0
        }));
    });

    console.log('\n🌐 网络请求分析:');
    networkEntries.forEach(req => {
      console.log(`- ${req.url}: ${req.duration}ms (${req.transferSize} bytes)`);
    });

    // 获取更多页面信息用于分析
    console.log('\n🔍 收集更多页面信息...');
    
    // 获取页面HTML结构分析
    const htmlStructure = await page.evaluate(() => {
      const info = {};
      
      // 检查React根元素
      const reactRoot = document.querySelector('#root');
      info.hasReactRoot = !!reactRoot;
      
      // 检查是否有加载错误
      const scripts = Array.from(document.querySelectorAll('script')).length;
      const stylesheets = Array.from(document.querySelectorAll('link[rel="stylesheet"]')).length;
      info.resourceCounts = { scripts, stylesheets };
      
      // 检查页面内容
      const bodyText = document.body.textContent || '';
      info.hasContent = bodyText.length > 100;
      info.containsAPI = bodyText.includes('API') || bodyText.includes('接口');
      info.containsError = bodyText.includes('错误') || bodyText.includes('Error') || bodyText.includes('error');
      
      return info;
    });
    
    console.log('📋 HTML结构分析:');
    console.log(`- React根元素存在: ${htmlStructure.hasReactRoot}`);
    console.log(`- 脚本数量: ${htmlStructure.resourceCounts.scripts}`);
    console.log(`- 样式表数量: ${htmlStructure.resourceCounts.stylesheets}`);
    console.log(`- 页面有内容: ${htmlStructure.hasContent}`);
    console.log(`- 包含API信息: ${htmlStructure.containsAPI}`);
    console.log(`- 包含错误信息: ${htmlStructure.containsError}`);

  } catch (error) {
    console.error('❌ 调试过程出错:', error.message);
    
    // 即使出错也要截图
    try {
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/screenshot-error.png',
        fullPage: true 
      });
      console.log('📸 错误状态截图已保存');
    } catch (screenshotError) {
      console.log('截图失败:', screenshotError.message);
    }
  } finally {
    await browser.close();
  }
}

debugDashboard().catch(console.error);