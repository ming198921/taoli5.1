import { chromium } from 'playwright';

async function testSystemControlFunctions() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🧪 开始测试系统控制功能...');
    
    // 监听控制台日志，捕获系统操作日志
    const systemLogs = [];
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('启动') || text.includes('停止') || text.includes('重启') || text.includes('🚀') || text.includes('🛑') || text.includes('🔄') || text.includes('🚨')) {
        systemLogs.push(text);
        console.log(`📋 ${text}`);
      }
    });

    // 访问系统页面
    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    console.log('⏰ 等待页面完全加载...');
    await page.waitForTimeout(5000);

    // 测试1：启动系统按钮
    console.log('\n🧪 测试1: 启动系统功能');
    const startButton = await page.$('button:has-text("启动系统")');
    if (startButton) {
      await startButton.click();
      console.log('✅ 启动系统按钮点击成功');
      
      // 等待操作完成
      await page.waitForTimeout(3000);
      
      // 检查是否有成功提示
      const successAlert = await page.$('.ant-modal, .ant-message');
      if (successAlert) {
        console.log('✅ 检测到操作提示框');
      }
    } else {
      console.log('❌ 未找到启动系统按钮');
    }

    // 等待并截图
    await page.waitForTimeout(2000);
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/test-start-system.png',
      fullPage: true 
    });

    // 测试2：停止系统按钮
    console.log('\n🧪 测试2: 停止系统功能');
    const stopButton = await page.$('button:has-text("停止系统")');
    if (stopButton) {
      await stopButton.click();
      console.log('✅ 停止系统按钮点击成功');
      await page.waitForTimeout(3000);
    } else {
      console.log('❌ 未找到停止系统按钮');
    }

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/test-stop-system.png',
      fullPage: true 
    });

    // 测试3：重启系统按钮
    console.log('\n🧪 测试3: 重启系统功能');
    const restartButton = await page.$('button:has-text("重启系统")');
    if (restartButton) {
      await restartButton.click();
      console.log('✅ 重启系统按钮点击成功');
      await page.waitForTimeout(5000); // 重启需要更长时间
    } else {
      console.log('❌ 未找到重启系统按钮');
    }

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/test-restart-system.png',
      fullPage: true 
    });

    // 测试4：紧急停止按钮
    console.log('\n🧪 测试4: 紧急停止功能');
    const emergencyButton = await page.$('button:has-text("紧急停止")');
    if (emergencyButton) {
      await emergencyButton.click();
      console.log('✅ 紧急停止按钮点击成功');
      await page.waitForTimeout(2000);
    } else {
      console.log('❌ 未找到紧急停止按钮');
    }

    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/test-emergency-stop.png',
      fullPage: true 
    });

    // 测试5：服务状态检查
    console.log('\n🧪 测试5: 验证服务状态数据');
    const serviceStatus = await page.evaluate(() => {
      const rows = Array.from(document.querySelectorAll('tbody tr'));
      return rows.map(row => {
        const cells = Array.from(row.querySelectorAll('td'));
        return {
          name: cells[0]?.textContent?.trim(),
          status: cells[1]?.textContent?.trim(),
          port: cells[2]?.textContent?.trim(),
          cpu: cells[4]?.textContent?.trim(),
          memory: cells[5]?.textContent?.trim()
        };
      }).filter(s => s.name);
    });

    console.log('📊 服务状态验证:');
    serviceStatus.forEach((service, index) => {
      const isHealthy = service.status?.includes('running');
      console.log(`${index + 1}. ${service.name}: ${service.status} (端口: ${service.port}) ${isHealthy ? '✅' : '❌'}`);
    });

    // 测试6：系统状态卡片检查
    const systemStats = await page.evaluate(() => {
      const statCards = Array.from(document.querySelectorAll('.ant-statistic-content-value'));
      return {
        status: document.querySelector('[class*="status"]:has-text("running")')?.textContent?.trim(),
        services: statCards[0]?.textContent?.trim(),
        uptime: statCards[1]?.textContent?.trim(),
        version: statCards[2]?.textContent?.trim()
      };
    });

    console.log('\n📊 系统状态卡片:');
    console.log(`- 系统状态: ${systemStats.status || '检测中'}`);
    console.log(`- 活跃服务: ${systemStats.services || '检测中'}`);
    console.log(`- 运行时间: ${systemStats.uptime || '检测中'}`);
    console.log(`- 系统版本: ${systemStats.version || '检测中'}`);

    // 汇总测试结果
    const hasRealData = serviceStatus.length >= 7;
    const allServicesHealthy = serviceStatus.every(s => s.status?.includes('running'));
    const buttonsWorking = systemLogs.length > 0;

    console.log('\n🏆 系统控制功能测试总结:');
    console.log(`- 检测到服务数量: ${serviceStatus.length}/7 ${serviceStatus.length >= 7 ? '✅' : '❌'}`);
    console.log(`- 所有服务健康: ${allServicesHealthy ? '✅' : '❌'}`);
    console.log(`- 控制按钮功能: ${buttonsWorking ? '✅' : '❌'}`);
    console.log(`- 系统日志记录: ${systemLogs.length}条`);

    if (systemLogs.length > 0) {
      console.log('\n📝 系统操作日志:');
      systemLogs.forEach((log, index) => {
        console.log(`${index + 1}. ${log}`);
      });
    }

    const overallSuccess = hasRealData && allServicesHealthy && buttonsWorking;
    console.log(`\n🎯 最终结果: ${overallSuccess ? '🎉 系统控制功能完全正常！' : '⚠️ 需要进一步调试'}`);

    return overallSuccess;

  } catch (error) {
    console.error('❌ 测试系统控制功能出错:', error.message);
    return false;
  } finally {
    await browser.close();
  }
}

testSystemControlFunctions().catch(console.error);