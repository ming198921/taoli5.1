import { chromium } from 'playwright';

async function debugSystemPage() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🚀 开始调试System页面...');
    
    // 监听所有网络请求
    const networkRequests = [];
    page.on('request', request => {
      if (request.url().includes('/api/') || request.url().includes('system')) {
        networkRequests.push({
          method: request.method(),
          url: request.url(),
          type: 'request'
        });
      }
    });

    page.on('response', response => {
      if (response.url().includes('/api/') || response.url().includes('system')) {
        const status = response.status();
        const url = response.url();
        if (status >= 400) {
          console.log(`❌ API Failed: ${status} ${url}`);
        } else {
          console.log(`✅ API Success: ${status} ${url}`);
        }
      }
    });

    // 监听控制台消息，重点关注系统控制相关
    page.on('console', msg => {
      const type = msg.type();
      const text = msg.text();
      if (type === 'error') {
        console.log(`❌ Console Error: ${text}`);
      } else if (type === 'warn') {
        console.log(`⚠️  Console Warning: ${text}`);
      } else if (text.includes('system') || text.includes('control') || text.includes('启动') || text.includes('停止')) {
        console.log(`📋 System Log: ${text}`);
      }
    });

    // 监听页面错误
    page.on('pageerror', error => {
      console.log(`💥 Page Error: ${error.message}`);
    });

    console.log('📖 访问System页面...');
    
    // 访问系统控制页面
    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    console.log('📸 截图分析页面初始状态...');
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/system-page-initial.png',
      fullPage: true 
    });

    // 等待页面加载
    console.log('⏰ 等待页面组件加载...');
    await page.waitForTimeout(5000);

    // 分析页面结构和按钮状态
    const pageAnalysis = await page.evaluate(() => {
      const analysis = {};
      
      // 检查页面标题和内容
      analysis.pageTitle = document.title;
      analysis.hasContent = document.body.textContent.length > 100;
      
      // 检查系统控制按钮
      const buttons = Array.from(document.querySelectorAll('button')).map(btn => ({
        text: btn.textContent?.trim(),
        disabled: btn.disabled,
        className: btn.className,
        visible: btn.offsetParent !== null
      }));
      analysis.buttons = buttons;
      
      // 检查是否有启动/停止相关的按钮
      analysis.hasStartButton = buttons.some(btn => 
        btn.text?.includes('启动') || btn.text?.includes('Start') || btn.text?.includes('开始')
      );
      analysis.hasStopButton = buttons.some(btn => 
        btn.text?.includes('停止') || btn.text?.includes('Stop') || btn.text?.includes('暂停')
      );
      analysis.hasRestartButton = buttons.some(btn => 
        btn.text?.includes('重启') || btn.text?.includes('Restart') || btn.text?.includes('重新')
      );
      analysis.hasEmergencyButton = buttons.some(btn => 
        btn.text?.includes('紧急') || btn.text?.includes('Emergency') || btn.text?.includes('强制')
      );
      
      // 检查加载状态
      const loadingElements = document.querySelectorAll('.ant-spin, [class*="loading"]');
      analysis.isLoading = loadingElements.length > 0;
      
      // 检查错误信息
      const errorElements = document.querySelectorAll('.ant-alert-error, [class*="error"]');
      analysis.errors = Array.from(errorElements).map(el => el.textContent);
      
      // 检查服务状态显示
      const statusCards = document.querySelectorAll('.ant-card, [class*="card"]');
      analysis.statusCardsCount = statusCards.length;
      
      return analysis;
    });

    console.log('\n📊 System页面分析结果:');
    console.log(`- 页面标题: ${pageAnalysis.pageTitle}`);
    console.log(`- 页面有内容: ${pageAnalysis.hasContent}`);
    console.log(`- 检测到按钮数量: ${pageAnalysis.buttons.length}`);
    console.log(`- 启动按钮: ${pageAnalysis.hasStartButton ? '✅' : '❌'}`);
    console.log(`- 停止按钮: ${pageAnalysis.hasStopButton ? '✅' : '❌'}`);  
    console.log(`- 重启按钮: ${pageAnalysis.hasRestartButton ? '✅' : '❌'}`);
    console.log(`- 紧急停止按钮: ${pageAnalysis.hasEmergencyButton ? '✅' : '❌'}`);
    console.log(`- 页面加载中: ${pageAnalysis.isLoading}`);
    console.log(`- 状态卡片数量: ${pageAnalysis.statusCardsCount}`);
    console.log(`- 错误信息数量: ${pageAnalysis.errors.length}`);

    if (pageAnalysis.errors.length > 0) {
      console.log('❌ 页面错误信息:');
      pageAnalysis.errors.forEach(error => console.log(`  - ${error}`));
    }

    console.log('\n🔘 按钮详细信息:');
    pageAnalysis.buttons.forEach((btn, index) => {
      console.log(`${index + 1}. "${btn.text}" (${btn.disabled ? '禁用' : '可用'}, ${btn.visible ? '可见' : '隐藏'})`);
    });

    // 尝试点击按钮测试API对接
    console.log('\n🧪 测试按钮API对接...');
    
    // 查找并测试启动按钮
    const startButton = await page.$('button:has-text("启动"), button:has-text("Start")');
    if (startButton) {
      console.log('🔄 测试启动按钮...');
      await startButton.click();
      await page.waitForTimeout(2000);
    } else {
      console.log('❌ 未找到启动按钮');
    }

    // 截图最终状态
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/system-page-final.png',
      fullPage: true 
    });

    console.log('\n🌐 网络请求汇总:');
    console.log(`- 总请求数: ${networkRequests.length}`);
    networkRequests.forEach(req => {
      console.log(`  - ${req.method} ${req.url}`);
    });

    // 检查是否存在系统控制API
    console.log('\n🔍 检查系统控制API可用性...');
    
  } catch (error) {
    console.error('❌ 调试System页面出错:', error.message);
    
    // 错误截图
    try {
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/system-page-error.png',
        fullPage: true 
      });
    } catch (e) {}
    
  } finally {
    await browser.close();
  }
}

debugSystemPage().catch(console.error);