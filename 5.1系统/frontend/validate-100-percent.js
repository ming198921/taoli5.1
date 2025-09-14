import { chromium } from 'playwright';

async function validate100PercentStability() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🚀 开始验证100%API稳定性...');
    
    // 监听所有API调用
    const apiCalls = [];
    page.on('response', response => {
      if (response.url().includes('/health') || response.url().includes('/api/')) {
        apiCalls.push({
          url: response.url(),
          status: response.status(),
          success: response.status() >= 200 && response.status() < 400
        });
      }
    });

    // 监听控制台数据更新
    let finalApiStats = null;
    let finalHealthData = null;
    
    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('apiStats') && text.includes('healthy')) {
        const match = text.match(/apiStats: ({.*})/);
        if (match) {
          try {
            finalApiStats = JSON.parse(match[1].replace(/(\w+):/g, '"$1":'));
          } catch (e) {
            // 解析失败，跳过
          }
        }
      }
      if (text.includes('获取到的服务健康状态')) {
        console.log(`📊 ${text}`);
      }
    });

    // 访问页面并强制刷新
    console.log('📖 访问Dashboard页面 (强制刷新缓存)...');
    await page.goto('http://57.183.21.242:3003/dashboard?t=' + Date.now(), { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    // 强制刷新页面
    await page.reload({ waitUntil: 'networkidle' });

    console.log('⏰ 等待所有API调用完成...');
    await page.waitForTimeout(8000);

    // 获取最终的页面数据
    const pageData = await page.evaluate(() => {
      const stats = {};
      
      // 获取统计数字
      const totalApis = document.querySelector('[data-testid*="total"], .ant-statistic-content-value')?.textContent;
      const healthyApis = document.querySelectorAll('.ant-statistic-content-value')[1]?.textContent;
      const responseTime = document.querySelectorAll('.ant-statistic-content-value')[2]?.textContent;
      const requestsPerSec = document.querySelectorAll('.ant-statistic-content-value')[3]?.textContent;
      
      // 获取服务状态
      const serviceRows = Array.from(document.querySelectorAll('tbody tr')).map(row => {
        const cells = Array.from(row.querySelectorAll('td'));
        return {
          name: cells[0]?.textContent?.trim(),
          status: cells[1]?.textContent?.trim(),
          apis: cells[2]?.textContent?.trim(),
          responseTime: cells[3]?.textContent?.trim(),
          uptime: cells[4]?.textContent?.trim()
        };
      });

      return {
        totalApis,
        healthyApis, 
        responseTime,
        requestsPerSec,
        serviceCount: serviceRows.length,
        services: serviceRows,
        hasData: serviceRows.length > 0 && serviceRows[0].name !== null
      };
    });

    console.log('\n📊 页面数据验证结果:');
    console.log(`- 总API接口: ${pageData.totalApis}`);
    console.log(`- 健康接口: ${pageData.healthyApis}`);
    console.log(`- 平均响应时间: ${pageData.responseTime}`);
    console.log(`- 请求/秒: ${pageData.requestsPerSec}`);
    console.log(`- 检测到服务数量: ${pageData.serviceCount}`);
    console.log(`- 是否有真实数据: ${pageData.hasData}`);

    console.log('\n🏥 服务详细状态:');
    pageData.services.forEach((service, index) => {
      const isReal = service.name && service.status && service.apis && service.responseTime;
      console.log(`${index + 1}. ${service.name}: ${service.status} (${service.apis} APIs, ${service.responseTime}, ${service.uptime}) ${isReal ? '✅ 真实数据' : '❌ 模拟数据'}`);
    });

    // 分析API调用成功率
    const successfulCalls = apiCalls.filter(call => call.success).length;
    const totalCalls = apiCalls.length;
    const successRate = totalCalls > 0 ? (successfulCalls / totalCalls) * 100 : 0;

    console.log('\n🌐 网络请求分析:');
    console.log(`- 总API调用: ${totalCalls}`);
    console.log(`- 成功调用: ${successfulCalls}`);
    console.log(`- API成功率: ${successRate.toFixed(1)}%`);

    // 验证是否达到100%稳定性
    const is100Percent = pageData.healthyApis === '387' || pageData.healthyApis?.includes('387');
    const hasAllServices = pageData.serviceCount === 7;
    const allServicesHealthy = pageData.services.every(s => s.status?.includes('健康') || s.status?.includes('正常'));

    console.log('\n🎯 100%稳定性验证:');
    console.log(`- 健康接口达到387个: ${is100Percent ? '✅' : '❌'}`);
    console.log(`- 检测到7个服务: ${hasAllServices ? '✅' : '❌'}`);
    console.log(`- 所有服务健康: ${allServicesHealthy ? '✅' : '❌'}`);
    console.log(`- 使用真实数据: ${pageData.hasData ? '✅' : '❌'}`);

    const isFullyStable = is100Percent && hasAllServices && allServicesHealthy && pageData.hasData;
    
    console.log(`\n🏆 最终结果: ${isFullyStable ? '🎉 已达到100%稳定性！' : '⚠️ 仍需优化'}`);

    // 截图验证
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/validation-screenshot.png',
      fullPage: true 
    });
    console.log('📸 验证截图已保存: validation-screenshot.png');

    return isFullyStable;

  } catch (error) {
    console.error('❌ 验证过程出错:', error.message);
    return false;
  } finally {
    await browser.close();
  }
}

validate100PercentStability().catch(console.error);