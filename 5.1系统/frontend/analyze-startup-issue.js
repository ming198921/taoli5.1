import { chromium } from 'playwright';

async function analyzeStartupIssue() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 开始分析启动按钮无反应问题...');
    
    // 详细监听所有事件
    const eventLogs = [];
    
    // 监听网络请求
    page.on('request', request => {
      eventLogs.push({
        type: 'network-request',
        method: request.method(),
        url: request.url(),
        timestamp: Date.now()
      });
    });

    page.on('response', response => {
      eventLogs.push({
        type: 'network-response',
        status: response.status(),
        url: response.url(),
        timestamp: Date.now()
      });
    });

    // 监听控制台输出
    page.on('console', msg => {
      eventLogs.push({
        type: 'console',
        level: msg.type(),
        text: msg.text(),
        timestamp: Date.now()
      });
    });

    // 监听页面错误
    page.on('pageerror', error => {
      eventLogs.push({
        type: 'page-error',
        message: error.message,
        timestamp: Date.now()
      });
    });

    console.log('📖 访问系统页面...');
    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(3000);

    // 第一步：分析页面当前状态
    console.log('\n📊 第一步：分析页面当前状态');
    const initialState = await page.evaluate(() => {
      // 获取所有按钮信息
      const buttons = Array.from(document.querySelectorAll('button')).map((btn, index) => ({
        index,
        text: btn.textContent?.trim(),
        disabled: btn.disabled,
        className: btn.className,
        onclick: btn.onclick !== null,
        eventListeners: btn.addEventListener !== undefined,
        visible: btn.offsetParent !== null,
        parentElement: btn.parentElement?.tagName
      }));

      // 找到启动按钮
      const startButton = buttons.find(btn => btn.text?.includes('启动系统'));
      
      // 检查React事件处理
      const reactEventHandlers = [];
      document.querySelectorAll('button').forEach((btn, index) => {
        const reactProps = Object.keys(btn).filter(key => key.startsWith('__reactEventHandlers'));
        if (reactProps.length > 0) {
          reactEventHandlers.push({ index, hasReactHandlers: true });
        }
      });

      return {
        totalButtons: buttons.length,
        startButton,
        allButtons: buttons,
        reactEventHandlers,
        modalCount: document.querySelectorAll('.ant-modal').length,
        timestamp: Date.now()
      };
    });

    console.log(`发现 ${initialState.totalButtons} 个按钮`);
    if (initialState.startButton) {
      console.log('启动系统按钮分析:');
      console.log(`  - 文本: "${initialState.startButton.text}"`);
      console.log(`  - 禁用状态: ${initialState.startButton.disabled}`);
      console.log(`  - 可见状态: ${initialState.startButton.visible}`);
      console.log(`  - CSS类: ${initialState.startButton.className}`);
      console.log(`  - 有onclick: ${initialState.startButton.onclick}`);
    }

    // 初始截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/analyze-initial.png',
      fullPage: true 
    });

    // 第二步：模拟点击并分析反应
    console.log('\n🎯 第二步：模拟点击启动按钮');
    const clickStartTime = Date.now();
    
    const startButton = await page.$('button:has-text("启动系统")');
    if (startButton) {
      // 在点击前记录事件日志数量
      const preClickEventCount = eventLogs.length;
      
      console.log('点击启动系统按钮...');
      await startButton.click();
      
      // 等待可能的反应
      await page.waitForTimeout(2000);
      
      const postClickEventCount = eventLogs.length;
      const newEvents = eventLogs.slice(preClickEventCount);
      
      console.log(`点击后新增事件数量: ${newEvents.length}`);
      
      // 分析点击后的DOM变化
      const postClickState = await page.evaluate(() => {
        return {
          modalCount: document.querySelectorAll('.ant-modal').length,
          modalVisible: document.querySelector('.ant-modal:not([style*="display: none"])')?.offsetParent !== null,
          modalContent: document.querySelector('.ant-modal-body')?.textContent?.trim(),
          confirmButtons: Array.from(document.querySelectorAll('.ant-modal .ant-btn')).map(btn => ({
            text: btn.textContent?.trim(),
            disabled: btn.disabled,
            primary: btn.className.includes('ant-btn-primary')
          })),
          messageCount: document.querySelectorAll('.ant-message').length,
          notificationCount: document.querySelectorAll('.ant-notification').length,
          timestamp: Date.now()
        };
      });

      console.log('点击后页面状态:');
      console.log(`  - 模态框数量: ${postClickState.modalCount}`);
      console.log(`  - 模态框可见: ${postClickState.modalVisible}`);
      console.log(`  - 模态框内容: "${postClickState.modalContent}"`);
      console.log(`  - 消息提示数量: ${postClickState.messageCount}`);
      console.log(`  - 通知数量: ${postClickState.notificationCount}`);
      
      if (postClickState.confirmButtons.length > 0) {
        console.log('  - 确认按钮:');
        postClickState.confirmButtons.forEach((btn, index) => {
          console.log(`    ${index + 1}. "${btn.text}" (${btn.primary ? '主要' : '次要'}, ${btn.disabled ? '禁用' : '可用'})`);
        });
      }

      // 点击后截图
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/analyze-after-click.png',
        fullPage: true 
      });

      // 第三步：如果有确认对话框，点击确认
      if (postClickState.modalCount > 0 && postClickState.confirmButtons.length > 0) {
        console.log('\n✅ 第三步：点击确认按钮');
        
        const confirmButton = await page.$('.ant-modal .ant-btn-primary');
        if (confirmButton) {
          const preConfirmEventCount = eventLogs.length;
          
          await confirmButton.click();
          console.log('已点击确认按钮');
          
          // 等待操作执行
          await page.waitForTimeout(5000);
          
          const postConfirmEventCount = eventLogs.length;
          const confirmEvents = eventLogs.slice(preConfirmEventCount);
          
          console.log(`确认后新增事件数量: ${confirmEvents.length}`);
          
          // 检查操作结果
          const finalState = await page.evaluate(() => {
            return {
              modalCount: document.querySelectorAll('.ant-modal').length,
              systemStatus: document.querySelector('[style*="color"]')?.textContent?.trim(),
              messageCount: document.querySelectorAll('.ant-message').length,
              timestamp: Date.now()
            };
          });

          console.log('确认后状态:');
          console.log(`  - 模态框数量: ${finalState.modalCount}`);
          console.log(`  - 系统状态: "${finalState.systemStatus}"`);
          console.log(`  - 消息数量: ${finalState.messageCount}`);

          // 最终截图
          await page.screenshot({ 
            path: '/home/ubuntu/arbitrage-frontend-v5.1/analyze-final.png',
            fullPage: true 
          });
        }
      } else {
        console.log('❌ 没有出现确认对话框');
      }
    } else {
      console.log('❌ 未找到启动系统按钮');
    }

    // 第四步：分析所有事件日志
    console.log('\n📋 第四步：事件日志分析');
    
    const networkRequests = eventLogs.filter(e => e.type === 'network-request');
    const networkResponses = eventLogs.filter(e => e.type === 'network-response');
    const consoleMessages = eventLogs.filter(e => e.type === 'console');
    const pageErrors = eventLogs.filter(e => e.type === 'page-error');
    
    console.log(`网络请求: ${networkRequests.length} 个`);
    console.log(`网络响应: ${networkResponses.length} 个`);
    console.log(`控制台消息: ${consoleMessages.length} 个`);
    console.log(`页面错误: ${pageErrors.length} 个`);

    // 分析系统相关的网络活动
    const systemRequests = networkRequests.filter(req => 
      req.url.includes('/system/') || req.url.includes('start') || req.url.includes('stop')
    );
    
    if (systemRequests.length > 0) {
      console.log('\n🌐 系统控制相关请求:');
      systemRequests.forEach(req => {
        console.log(`  - ${req.method} ${req.url}`);
      });
    } else {
      console.log('\n❌ 未发现系统控制相关的网络请求');
    }

    // 分析控制台中的系统日志
    const systemLogs = consoleMessages.filter(msg => 
      msg.text.includes('启动') || msg.text.includes('系统') || 
      msg.text.includes('🚀') || msg.text.includes('start')
    );
    
    if (systemLogs.length > 0) {
      console.log('\n📝 系统操作相关日志:');
      systemLogs.forEach(log => {
        console.log(`  - [${log.level}] ${log.text}`);
      });
    } else {
      console.log('\n❌ 未发现系统操作相关的控制台日志');
    }

    // 检查是否有错误
    if (pageErrors.length > 0) {
      console.log('\n❗ 页面错误:');
      pageErrors.forEach(error => {
        console.log(`  - ${error.message}`);
      });
    }

    // 总结诊断结果
    console.log('\n🎯 诊断结果总结:');
    const hasModal = postClickState?.modalCount > 0;
    const hasSystemRequests = systemRequests.length > 0;
    const hasSystemLogs = systemLogs.length > 0;
    const hasErrors = pageErrors.length > 0;
    
    console.log(`✅ 按钮可点击: ${initialState.startButton?.visible && !initialState.startButton?.disabled}`);
    console.log(`✅ 出现确认框: ${hasModal}`);
    console.log(`✅ 发起API请求: ${hasSystemRequests}`);
    console.log(`✅ 产生系统日志: ${hasSystemLogs}`);
    console.log(`❌ 出现错误: ${hasErrors}`);

    if (!hasModal) {
      console.log('\n🔧 问题分析: 启动按钮点击后没有出现确认对话框');
      console.log('可能原因:');
      console.log('1. 按钮事件处理器未正确绑定');
      console.log('2. React组件状态问题');
      console.log('3. JavaScript错误中断了执行');
    } else if (!hasSystemRequests) {
      console.log('\n🔧 问题分析: 确认对话框出现但未发起API请求');
      console.log('可能原因:');
      console.log('1. 确认按钮的事件处理有问题');
      console.log('2. API调用被阻止或失败');
      console.log('3. 网络连接问题');
    }

  } catch (error) {
    console.error('❌ 分析过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

analyzeStartupIssue().catch(console.error);