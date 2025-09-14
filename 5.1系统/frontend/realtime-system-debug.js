import { chromium } from 'playwright';

async function realtimeSystemDebug() {
  const browser = await chromium.launch({ 
    headless: false,  // 显示浏览器窗口
    devtools: true,   // 打开开发者工具
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🚀 启动实时系统页面调试...');
    
    // 监听所有网络请求和响应
    const networkLogs = [];
    page.on('request', request => {
      if (request.url().includes('system') || request.url().includes('api')) {
        networkLogs.push({
          type: 'request',
          method: request.method(),
          url: request.url(),
          timestamp: new Date().toISOString()
        });
        console.log(`📤 API Request: ${request.method()} ${request.url()}`);
      }
    });

    page.on('response', response => {
      if (response.url().includes('system') || response.url().includes('api')) {
        const status = response.status();
        networkLogs.push({
          type: 'response',
          status,
          url: response.url(),
          timestamp: new Date().toISOString()
        });
        console.log(`📥 API Response: ${status} ${response.url()}`);
      }
    });

    // 监听控制台消息
    const consoleLogs = [];
    page.on('console', msg => {
      const text = msg.text();
      consoleLogs.push({
        type: msg.type(),
        text,
        timestamp: new Date().toISOString()
      });
      
      if (msg.type() === 'error') {
        console.log(`❌ Console Error: ${text}`);
      } else if (text.includes('启动') || text.includes('系统') || text.includes('🚀')) {
        console.log(`📋 System Log: ${text}`);
      }
    });

    // 访问系统页面
    console.log('📖 访问System页面...');
    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    console.log('⏰ 等待页面完全加载...');
    await page.waitForTimeout(3000);

    // 初始页面截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/realtime-initial.png',
      fullPage: true 
    });
    console.log('📸 初始页面截图已保存: realtime-initial.png');

    // 分析启动按钮状态
    const buttonAnalysis = await page.evaluate(() => {
      const buttons = Array.from(document.querySelectorAll('button'));
      const startButton = buttons.find(btn => btn.textContent?.includes('启动系统'));
      
      return {
        totalButtons: buttons.length,
        startButton: startButton ? {
          text: startButton.textContent?.trim(),
          disabled: startButton.disabled,
          className: startButton.className,
          visible: startButton.offsetParent !== null,
          clickable: !startButton.disabled && startButton.offsetParent !== null
        } : null,
        allButtons: buttons.map(btn => ({
          text: btn.textContent?.trim(),
          disabled: btn.disabled,
          visible: btn.offsetParent !== null
        }))
      };
    });

    console.log('\n🔍 启动按钮分析:');
    if (buttonAnalysis.startButton) {
      console.log(`- 按钮文本: "${buttonAnalysis.startButton.text}"`);
      console.log(`- 是否禁用: ${buttonAnalysis.startButton.disabled}`);
      console.log(`- 是否可见: ${buttonAnalysis.startButton.visible}`);
      console.log(`- 是否可点击: ${buttonAnalysis.startButton.clickable}`);
      console.log(`- CSS类名: ${buttonAnalysis.startButton.className}`);
    } else {
      console.log('❌ 未找到启动系统按钮');
    }

    console.log('\n🧪 开始测试启动按钮点击...');
    
    // 点击启动按钮前的状态
    const preClickState = await page.evaluate(() => {
      return {
        modalCount: document.querySelectorAll('.ant-modal').length,
        timestamp: new Date().toISOString()
      };
    });
    console.log(`点击前状态 - 模态框数量: ${preClickState.modalCount}`);

    // 点击启动按钮
    const startButton = await page.$('button:has-text("启动系统")');
    if (startButton) {
      console.log('🎯 点击启动系统按钮...');
      await startButton.click();
      
      // 等待可能的模态框出现
      await page.waitForTimeout(1000);
      
      // 检查点击后的页面状态
      const postClickState = await page.evaluate(() => {
        return {
          modalCount: document.querySelectorAll('.ant-modal').length,
          modalVisible: document.querySelector('.ant-modal')?.style?.display !== 'none',
          modalContent: document.querySelector('.ant-modal-body')?.textContent?.trim(),
          confirmButton: document.querySelector('.ant-modal .ant-btn-primary')?.textContent?.trim(),
          cancelButton: document.querySelector('.ant-modal .ant-btn:not(.ant-btn-primary)')?.textContent?.trim(),
          timestamp: new Date().toISOString()
        };
      });

      console.log('\n📊 点击后状态分析:');
      console.log(`- 模态框数量: ${postClickState.modalCount}`);
      console.log(`- 模态框可见: ${postClickState.modalVisible}`);
      console.log(`- 模态框内容: "${postClickState.modalContent}"`);
      console.log(`- 确认按钮: "${postClickState.confirmButton}"`);
      console.log(`- 取消按钮: "${postClickState.cancelButton}"`);

      // 截图显示点击后状态
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/realtime-after-click.png',
        fullPage: true 
      });
      console.log('📸 点击后截图已保存: realtime-after-click.png');

      // 如果有确认对话框，点击确认
      if (postClickState.modalCount > 0 && postClickState.confirmButton) {
        console.log('🎯 点击确认按钮...');
        const confirmBtn = await page.$('.ant-modal .ant-btn-primary');
        if (confirmBtn) {
          await confirmBtn.click();
          console.log('✅ 已点击确认按钮');
          
          // 等待启动操作执行
          console.log('⏳ 等待启动操作执行...');
          await page.waitForTimeout(5000);
          
          // 检查启动是否成功
          const postConfirmState = await page.evaluate(() => {
            return {
              modalCount: document.querySelectorAll('.ant-modal').length,
              systemStatus: document.querySelector('[style*="color"]')?.textContent?.trim(),
              timestamp: new Date().toISOString()
            };
          });

          console.log('\n📈 启动操作执行后状态:');
          console.log(`- 模态框数量: ${postConfirmState.modalCount}`);
          console.log(`- 系统状态: "${postConfirmState.systemStatus}"`);
          
          // 最终截图
          await page.screenshot({ 
            path: '/home/ubuntu/arbitrage-frontend-v5.1/realtime-final.png',
            fullPage: true 
          });
          console.log('📸 最终状态截图已保存: realtime-final.png');
        }
      }
    } else {
      console.log('❌ 未找到启动系统按钮，无法进行点击测试');
    }

    // 总结网络请求
    console.log('\n🌐 网络请求总结:');
    console.log(`- 总请求数: ${networkLogs.filter(log => log.type === 'request').length}`);
    console.log(`- 总响应数: ${networkLogs.filter(log => log.type === 'response').length}`);
    
    const systemRequests = networkLogs.filter(log => 
      log.url?.includes('/system/') && log.type === 'request'
    );
    console.log(`- 系统控制API请求: ${systemRequests.length}`);
    
    systemRequests.forEach(req => {
      console.log(`  - ${req.method} ${req.url}`);
    });

    // 总结控制台日志
    const errorLogs = consoleLogs.filter(log => log.type === 'error');
    const systemLogs = consoleLogs.filter(log => 
      log.text.includes('启动') || log.text.includes('系统') || log.text.includes('🚀')
    );
    
    console.log('\n📝 控制台日志总结:');
    console.log(`- 错误日志: ${errorLogs.length}条`);
    console.log(`- 系统操作日志: ${systemLogs.length}条`);
    
    if (systemLogs.length > 0) {
      console.log('系统操作日志详情:');
      systemLogs.forEach((log, index) => {
        console.log(`  ${index + 1}. ${log.text}`);
      });
    }

    console.log('\n🎯 启动操作分析结论:');
    const hasModal = postClickState?.modalCount > 0;
    const hasSystemLogs = systemLogs.length > 0;
    const hasNetworkActivity = systemRequests.length > 0;
    
    console.log(`- 点击响应: ${hasModal ? '✅ 显示确认对话框' : '❌ 无反应'}`);
    console.log(`- 系统日志: ${hasSystemLogs ? '✅ 有启动日志' : '❌ 无启动日志'}`);
    console.log(`- 网络请求: ${hasNetworkActivity ? '✅ 有API调用' : '❌ 无API调用'}`);

    // 保持浏览器打开以供实时查看
    console.log('\n🔍 浏览器保持打开状态，可实时查看页面和开发者工具');
    console.log('按 Ctrl+C 结束调试');
    
    // 等待用户中断
    await new Promise(resolve => {
      process.on('SIGINT', resolve);
    });

  } catch (error) {
    console.error('❌ 实时调试出错:', error.message);
  } finally {
    console.log('🔚 结束调试，关闭浏览器');
    await browser.close();
  }
}

realtimeSystemDebug().catch(console.error);