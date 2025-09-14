import { chromium } from 'playwright';

async function testUserFeedback() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🧪 测试改进后的用户反馈功能...');
    
    // 监听通知和消息
    const uiEvents = [];
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('启动成功') || text.includes('notification') || text.includes('message') || text.includes('🚀')) {
        uiEvents.push({
          type: 'console',
          text,
          timestamp: Date.now()
        });
      }
    });

    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(3000);

    // 初始截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/feedback-test-initial.png',
      fullPage: true 
    });

    console.log('🎯 点击启动系统按钮...');
    const startButton = await page.$('button:has-text("启动系统")');
    if (startButton) {
      await startButton.click();
      
      // 等待确认对话框
      await page.waitForSelector('.ant-modal', { timeout: 3000 });
      console.log('✅ 确认对话框出现');
      
      // 截图显示确认对话框
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/feedback-test-modal.png',
        fullPage: true 
      });
      
      // 点击确认
      const confirmButton = await page.$('.ant-modal .ant-btn-primary');
      if (confirmButton) {
        console.log('🎯 点击确认按钮...');
        await confirmButton.click();
        
        // 等待操作完成和反馈显示
        console.log('⏳ 等待操作反馈...');
        await page.waitForTimeout(3000);
        
        // 检查是否有加载提示
        const loadingElements = await page.$$('.ant-message-loading');
        console.log(`📋 加载提示数量: ${loadingElements.length}`);
        
        // 等待更长时间以确保通知显示
        await page.waitForTimeout(3000);
        
        // 检查通知
        const notificationElements = await page.$$('.ant-notification');
        const messageElements = await page.$$('.ant-message');
        
        console.log(`📢 通知数量: ${notificationElements.length}`);
        console.log(`📨 消息数量: ${messageElements.length}`);
        
        // 获取通知内容
        if (notificationElements.length > 0) {
          const notificationContent = await page.evaluate(() => {
            const notifications = Array.from(document.querySelectorAll('.ant-notification'));
            return notifications.map(notif => ({
              title: notif.querySelector('.ant-notification-notice-message')?.textContent?.trim(),
              description: notif.querySelector('.ant-notification-notice-description')?.textContent?.trim(),
              type: notif.className.includes('success') ? 'success' : 
                    notif.className.includes('error') ? 'error' : 
                    notif.className.includes('warning') ? 'warning' : 'info'
            }));
          });
          
          console.log('📋 通知内容:');
          notificationContent.forEach((notif, index) => {
            console.log(`  ${index + 1}. [${notif.type}] ${notif.title}: ${notif.description}`);
          });
        }
        
        // 获取消息内容
        if (messageElements.length > 0) {
          const messageContent = await page.evaluate(() => {
            const messages = Array.from(document.querySelectorAll('.ant-message'));
            return messages.map(msg => msg.textContent?.trim());
          });
          
          console.log('📨 消息内容:');
          messageContent.forEach((msg, index) => {
            console.log(`  ${index + 1}. ${msg}`);
          });
        }
        
        // 最终截图
        await page.screenshot({ 
          path: '/home/ubuntu/arbitrage-frontend-v5.1/feedback-test-final.png',
          fullPage: true 
        });
        
        // 检查系统状态是否有变化
        const systemStatus = await page.evaluate(() => {
          return {
            status: document.querySelector('[style*="color"]')?.textContent?.trim(),
            services: document.querySelector('div').textContent?.includes('7/7'),
            timestamp: Date.now()
          };
        });
        
        console.log('\n📊 操作后系统状态:');
        console.log(`- 系统状态: ${systemStatus.status}`);
        console.log(`- 服务状态: ${systemStatus.services ? '7/7 正常' : '未检测到'}`);
      }
    }

    // 总结测试结果
    console.log('\n🎯 用户反馈测试总结:');
    const hasNotifications = notificationElements?.length > 0;
    const hasMessages = messageElements?.length > 0;
    const hasUIEvents = uiEvents.length > 0;
    
    console.log(`✅ 确认对话框: 正常显示`);
    console.log(`✅ 通知反馈: ${hasNotifications ? '显示正常' : '未显示'}`);
    console.log(`✅ 消息反馈: ${hasMessages ? '显示正常' : '未显示'}`);
    console.log(`✅ 控制台事件: ${hasUIEvents ? `${uiEvents.length}个事件` : '无事件'}`);
    
    const userExperienceGood = hasNotifications || hasMessages || hasUIEvents;
    console.log(`\n🏆 用户体验评估: ${userExperienceGood ? '🎉 反馈充分，体验良好' : '⚠️ 反馈不足，需要改进'}`);

    if (uiEvents.length > 0) {
      console.log('\n📝 UI事件详情:');
      uiEvents.forEach((event, index) => {
        console.log(`${index + 1}. ${event.text}`);
      });
    }

  } catch (error) {
    console.error('❌ 用户反馈测试出错:', error.message);
  } finally {
    await browser.close();
  }
}

testUserFeedback().catch(console.error);