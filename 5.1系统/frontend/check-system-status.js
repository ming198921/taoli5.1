import { chromium } from 'playwright';

async function checkSystemStatus() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 检查系统控制页面当前状态...');
    
    // 监听控制台消息
    page.on('console', msg => {
      if (msg.type() === 'error' || msg.text().includes('❌') || msg.text().includes('⚠️')) {
        console.log(`Console: ${msg.text()}`);
      }
    });

    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(5000); // 等待数据加载

    // 检查系统状态卡片
    const systemOverview = await page.evaluate(() => {
      const cards = document.querySelectorAll('.ant-card');
      const statusCards = [];
      
      cards.forEach(card => {
        const content = card.textContent;
        if (content.includes('系统状态') || content.includes('运行服务') || content.includes('运行时间') || content.includes('系统版本')) {
          statusCards.push({
            text: content.replace(/\s+/g, ' ').trim(),
            hasData: !content.includes('0') || content.includes('running') || content.includes('v5.1.0')
          });
        }
      });
      
      return statusCards;
    });

    console.log('\n📊 系统状态概览:');
    systemOverview.forEach((card, index) => {
      console.log(`${index + 1}. ${card.hasData ? '✅' : '❌'} ${card.text}`);
    });

    // 检查服务管理表格
    const serviceTable = await page.evaluate(() => {
      const table = document.querySelector('.ant-table-tbody');
      const rows = table?.querySelectorAll('tr') || [];
      const services = [];
      
      rows.forEach(row => {
        const cells = row.querySelectorAll('td');
        if (cells.length >= 6) {
          services.push({
            name: cells[0]?.textContent?.trim(),
            status: cells[1]?.textContent?.trim(),
            port: cells[2]?.textContent?.trim(),
            cpu: cells[4]?.textContent?.trim(),
            memory: cells[5]?.textContent?.trim()
          });
        }
      });
      
      return {
        serviceCount: services.length,
        services: services.slice(0, 3), // 显示前3个
        allRunning: services.every(s => s.status?.includes('running'))
      };
    });

    console.log('\n📋 服务管理状态:');
    console.log(`服务总数: ${serviceTable.serviceCount}`);
    console.log(`全部运行: ${serviceTable.allRunning ? '✅' : '❌'}`);
    if (serviceTable.services.length > 0) {
      console.log('前3个服务:');
      serviceTable.services.forEach(service => {
        console.log(`  - ${service.name}: ${service.status} (${service.cpu}, ${service.memory})`);
      });
    }

    // 检查系统监控
    const monitoringTab = await page.$('div[role="tab"]:has-text("系统监控")');
    if (monitoringTab) {
      await monitoringTab.click();
      await page.waitForTimeout(2000);
      
      const monitoringData = await page.evaluate(() => {
        const progressBars = document.querySelectorAll('.ant-progress-text');
        const progressValues = Array.from(progressBars).map(el => el.textContent);
        
        const networkItems = document.querySelectorAll('[class*="ant-badge"]');
        const networkCount = Array.from(networkItems).filter(item => 
          item.parentElement?.textContent?.includes('网关') || 
          item.parentElement?.textContent?.includes('API') ||
          item.parentElement?.textContent?.includes('WebSocket') ||
          item.parentElement?.textContent?.includes('负载均衡')
        ).length;
        
        return {
          cpuValue: progressValues[0] || 'N/A',
          memoryValue: progressValues[1] || 'N/A', 
          diskValue: progressValues[2] || 'N/A',
          networkItems: networkCount,
          isDynamic: !progressValues.includes('65%') && !progressValues.includes('58%') && !progressValues.includes('42%')
        };
      });

      console.log('\n📈 系统监控数据:');
      console.log(`CPU使用率: ${monitoringData.cpuValue}`);
      console.log(`内存使用: ${monitoringData.memoryValue}`);
      console.log(`磁盘使用: ${monitoringData.diskValue}`);
      console.log(`网络状态项: ${monitoringData.networkItems}`);
      console.log(`数据动态: ${monitoringData.isDynamic ? '✅ 是' : '❌ 否(固定数据)'}`);
    }

    // 检查系统诊断
    const diagnosticsTab = await page.$('div[role="tab"]:has-text("系统诊断")');
    if (diagnosticsTab) {
      await diagnosticsTab.click();
      await page.waitForTimeout(2000);
      
      const runButton = await page.$('button:has-text("运行诊断")');
      if (runButton) {
        await runButton.click();
        await page.waitForTimeout(3000);
        
        const diagnosticsResults = await page.evaluate(() => {
          const alerts = document.querySelectorAll('.ant-alert');
          const results = Array.from(alerts).map(alert => ({
            type: alert.className,
            message: alert.querySelector('.ant-alert-message')?.textContent?.trim()
          }));
          
          return {
            count: alerts.length,
            results: results.slice(0, 2),
            hasResults: alerts.length > 0
          };
        });
        
        console.log('\n🔧 系统诊断结果:');
        console.log(`诊断项数: ${diagnosticsResults.count}`);
        console.log(`有结果: ${diagnosticsResults.hasResults ? '✅' : '❌'}`);
        if (diagnosticsResults.results.length > 0) {
          diagnosticsResults.results.forEach((result, index) => {
            console.log(`  ${index + 1}. ${result.message}`);
          });
        }
      }
    }

    // 最终截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/current-system-status.png',
      fullPage: true 
    });

    console.log('\n🎯 系统状态检查完成！');

  } catch (error) {
    console.error('❌ 检查过程出错:', error.message);
  } finally {
    await browser.close();
  }
}

checkSystemStatus().catch(console.error);