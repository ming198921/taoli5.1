import { chromium } from 'playwright';

async function detailedIssueAnalysis() {
  const browser = await chromium.launch({ 
    headless: true,
    args: ['--no-sandbox', '--disable-dev-shm-usage']
  });
  
  try {
    const context = await browser.newContext();
    const page = await context.newPage();
    
    console.log('🔍 开始详细问题分析...');
    
    // 监听所有网络活动和UI状态
    const networkActivity = [];
    const uiStateChanges = [];
    
    page.on('request', request => {
      networkActivity.push({
        type: 'request',
        method: request.method(),
        url: request.url(),
        timestamp: new Date().toISOString()
      });
    });

    page.on('response', response => {
      networkActivity.push({
        type: 'response',
        status: response.status(),
        url: response.url(),
        timestamp: new Date().toISOString()
      });
    });

    page.on('console', msg => {
      const text = msg.text();
      if (text.includes('loading') || text.includes('转圈') || text.includes('更新') || text.includes('诊断')) {
        uiStateChanges.push({
          type: 'console',
          text,
          timestamp: new Date().toISOString()
        });
      }
    });

    await page.goto('http://57.183.21.242:3003/system', { 
      waitUntil: 'networkidle',
      timeout: 30000 
    });

    await page.waitForTimeout(3000);

    console.log('\n📊 问题1: 分析启动确认后的转圈和反馈问题');
    
    // 初始状态截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-initial.png',
      fullPage: true 
    });

    // 检查初始加载状态
    const initialLoadingState = await page.evaluate(() => {
      return {
        loadingSpinners: document.querySelectorAll('.ant-spin-spinning').length,
        loadingButtons: document.querySelectorAll('.ant-btn-loading').length,
        tableLoading: document.querySelector('.ant-table-tbody')?.textContent?.includes('loading'),
        emptyStates: document.querySelectorAll('.ant-empty').length,
        timestamp: new Date().toISOString()
      };
    });

    console.log('初始加载状态:');
    console.log(`- 加载中的转圈组件: ${initialLoadingState.loadingSpinners}`);
    console.log(`- 加载中的按钮: ${initialLoadingState.loadingButtons}`);
    console.log(`- 表格加载状态: ${initialLoadingState.tableLoading}`);
    console.log(`- 空状态组件: ${initialLoadingState.emptyStates}`);

    // 测试启动按钮的完整流程
    console.log('\n🧪 测试启动按钮完整流程...');
    const startButton = await page.$('button:has-text("启动系统")');
    
    if (startButton) {
      // 点击启动按钮
      await startButton.click();
      console.log('✅ 点击启动按钮');
      
      // 等待确认对话框
      await page.waitForSelector('.ant-modal', { timeout: 3000 });
      
      // 确认对话框截图
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-modal.png',
        fullPage: true 
      });
      
      // 点击确认按钮
      const confirmButton = await page.$('.ant-modal .ant-btn-primary');
      if (confirmButton) {
        console.log('🎯 点击确认按钮...');
        await confirmButton.click();
        
        // 立即检查加载状态
        const postClickState = await page.evaluate(() => {
          return {
            loadingMessages: document.querySelectorAll('.ant-message-loading').length,
            loadingSpinners: document.querySelectorAll('.ant-spin-spinning').length,
            modalVisible: document.querySelectorAll('.ant-modal:not([style*="display: none"])').length,
            timestamp: new Date().toISOString()
          };
        });
        
        console.log('点击确认后立即状态:');
        console.log(`- 加载消息: ${postClickState.loadingMessages}`);
        console.log(`- 转圈组件: ${postClickState.loadingSpinners}`);
        console.log(`- 可见模态框: ${postClickState.modalVisible}`);
        
        // 等待并持续监控状态变化
        for (let i = 1; i <= 10; i++) {
          await page.waitForTimeout(1000);
          
          const currentState = await page.evaluate(() => {
            return {
              loadingMessages: document.querySelectorAll('.ant-message-loading').length,
              notifications: document.querySelectorAll('.ant-notification').length,
              spinners: document.querySelectorAll('.ant-spin-spinning').length,
              systemStatus: document.querySelector('[style*="color"]')?.textContent?.trim()
            };
          });
          
          console.log(`${i}秒后状态: 加载消息:${currentState.loadingMessages}, 通知:${currentState.notifications}, 转圈:${currentState.spinners}, 状态:${currentState.systemStatus}`);
          
          // 如果检测到变化，截图记录
          if (currentState.notifications > 0 || currentState.loadingMessages === 0) {
            await page.screenshot({ 
              path: `/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-${i}s.png`,
              fullPage: true 
            });
            break;
          }
        }
      }
    }

    console.log('\n📊 问题2: 分析服务管理信息转圈问题');
    
    // 检查服务管理表格状态
    const serviceTableState = await page.evaluate(() => {
      const table = document.querySelector('.ant-table-tbody');
      const rows = document.querySelectorAll('.ant-table-tbody tr');
      const loadingRows = document.querySelectorAll('.ant-table-tbody .ant-spin');
      const emptyState = document.querySelector('.ant-empty');
      
      return {
        hasTable: !!table,
        rowCount: rows.length,
        loadingRowCount: loadingRows.length,
        hasEmptyState: !!emptyState,
        tableContent: table?.textContent?.slice(0, 200),
        timestamp: new Date().toISOString()
      };
    });

    console.log('服务管理表格状态:');
    console.log(`- 表格存在: ${serviceTableState.hasTable}`);
    console.log(`- 数据行数: ${serviceTableState.rowCount}`);
    console.log(`- 加载中行数: ${serviceTableState.loadingRowCount}`);
    console.log(`- 空状态: ${serviceTableState.hasEmptyState}`);
    console.log(`- 表格内容预览: ${serviceTableState.tableContent}`);

    console.log('\n📊 问题3: 分析系统监控数据');
    
    // 切换到系统监控选项卡
    const monitoringTab = await page.$('div[role="tab"]:has-text("系统监控")');
    if (monitoringTab) {
      await monitoringTab.click();
      await page.waitForTimeout(2000);
      
      const monitoringData = await page.evaluate(() => {
        const progressBars = Array.from(document.querySelectorAll('.ant-progress'));
        const progressValues = progressBars.map(bar => {
          const percentElement = bar.querySelector('.ant-progress-text');
          return percentElement?.textContent?.trim();
        });
        
        const networkStatus = Array.from(document.querySelectorAll('.ant-badge')).map(badge => ({
          text: badge.parentElement?.textContent?.trim(),
          status: badge.className
        }));
        
        return {
          progressValues,
          networkStatus,
          timestamp: new Date().toISOString()
        };
      });
      
      console.log('系统监控数据:');
      console.log('- 进度条数值:', monitoringData.progressValues);
      console.log('- 网络状态:', monitoringData.networkStatus);
      
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-monitoring.png',
        fullPage: true 
      });
    }

    console.log('\n📊 问题4: 分析系统诊断功能');
    
    // 切换到系统诊断选项卡
    const diagnosticsTab = await page.$('div[role="tab"]:has-text("系统诊断")');
    if (diagnosticsTab) {
      await diagnosticsTab.click();
      await page.waitForTimeout(2000);
      
      // 检查诊断按钮和内容
      const diagnosticsState = await page.evaluate(() => {
        const runButton = document.querySelector('button:has-text("运行诊断")');
        const diagnosticsContent = document.querySelector('.ant-card .ant-card-body');
        const alerts = document.querySelectorAll('.ant-alert');
        
        return {
          hasRunButton: !!runButton,
          buttonDisabled: runButton?.disabled,
          contentEmpty: !diagnosticsContent?.textContent?.trim() || diagnosticsContent?.textContent?.includes('点击'),
          alertCount: alerts.length,
          contentText: diagnosticsContent?.textContent?.trim(),
          timestamp: new Date().toISOString()
        };
      });
      
      console.log('系统诊断状态:');
      console.log(`- 运行按钮存在: ${diagnosticsState.hasRunButton}`);
      console.log(`- 按钮禁用: ${diagnosticsState.buttonDisabled}`);
      console.log(`- 内容为空: ${diagnosticsState.contentEmpty}`);
      console.log(`- 警告数量: ${diagnosticsState.alertCount}`);
      console.log(`- 内容文本: ${diagnosticsState.contentText}`);
      
      // 尝试点击运行诊断按钮
      const runDiagnosticsButton = await page.$('button:has-text("运行诊断")');
      if (runDiagnosticsButton) {
        console.log('🎯 点击运行诊断按钮...');
        await runDiagnosticsButton.click();
        await page.waitForTimeout(3000);
        
        const postDiagnosticsState = await page.evaluate(() => {
          const alerts = Array.from(document.querySelectorAll('.ant-alert')).map(alert => ({
            type: alert.className,
            message: alert.querySelector('.ant-alert-message')?.textContent?.trim(),
            description: alert.querySelector('.ant-alert-description')?.textContent?.trim()
          }));
          
          return {
            alertCount: alerts.length,
            alerts,
            timestamp: new Date().toISOString()
          };
        });
        
        console.log('诊断运行后状态:');
        console.log(`- 诊断结果数量: ${postDiagnosticsState.alertCount}`);
        if (postDiagnosticsState.alerts.length > 0) {
          postDiagnosticsState.alerts.forEach((alert, index) => {
            console.log(`  ${index + 1}. ${alert.message}: ${alert.description}`);
          });
        }
      }
      
      await page.screenshot({ 
        path: '/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-diagnostics.png',
        fullPage: true 
      });
    }

    // 分析网络请求模式
    console.log('\n🌐 网络活动分析:');
    const systemRequests = networkActivity.filter(activity => 
      activity.url.includes('/system/') || activity.url.includes('/health')
    );
    
    console.log(`系统相关请求总数: ${systemRequests.filter(req => req.type === 'request').length}`);
    console.log(`系统相关响应总数: ${systemRequests.filter(req => req.type === 'response').length}`);
    
    const failedRequests = systemRequests.filter(req => 
      req.type === 'response' && req.status >= 400
    );
    
    if (failedRequests.length > 0) {
      console.log('\n❌ 失败的请求:');
      failedRequests.forEach(req => {
        console.log(`  - ${req.status} ${req.url}`);
      });
    }

    // 最终总结截图
    await page.screenshot({ 
      path: '/home/ubuntu/arbitrage-frontend-v5.1/issue-analysis-final.png',
      fullPage: true 
    });

    console.log('\n🎯 问题分析总结:');
    console.log('1. 启动确认后转圈问题: 需要检查loading状态管理');
    console.log('2. 服务管理信息转圈: 需要检查数据获取和渲染逻辑');
    console.log('3. 系统监控固定数据: 需要连接真实监控数据源');
    console.log('4. 系统状态更新: 需要实现实时更新机制');
    console.log('5. 系统诊断为空: 需要实现诊断逻辑或fallback数据');

  } catch (error) {
    console.error('❌ 问题分析出错:', error.message);
  } finally {
    await browser.close();
  }
}

detailedIssueAnalysis().catch(console.error);