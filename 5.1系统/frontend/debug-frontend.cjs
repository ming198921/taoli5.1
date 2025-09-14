#!/usr/bin/env node

const puppeteer = require('puppeteer');

async function debugFrontend() {
    let browser;
    try {
        console.log('🚀 启动浏览器调试...');
        
        browser = await puppeteer.launch({
            headless: true,
            args: ['--no-sandbox', '--disable-dev-shm-usage']
        });
        
        const page = await browser.newPage();
        
        // 监听控制台消息
        page.on('console', msg => {
            const type = msg.type();
            if (type === 'error') {
                console.log(`❌ JS Error: ${msg.text()}`);
            } else if (type === 'warn') {
                console.log(`⚠️  JS Warning: ${msg.text()}`);
            } else if (type === 'log') {
                console.log(`📋 JS Log: ${msg.text()}`);
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

        console.log('📖 正在访问Dashboard页面...');
        
        // 设置更长的超时时间
        await page.goto('http://localhost:3003/dashboard', {
            waitUntil: 'networkidle0',
            timeout: 30000
        });

        console.log('⏰ 等待页面完全加载...');
        await page.waitForTimeout(5000);

        // 检查页面标题
        const title = await page.title();
        console.log(`📄 页面标题: ${title}`);

        // 检查是否有加载错误的组件
        const errorElements = await page.$$eval('[class*="error"], [class*="Error"], .ant-alert-error', 
            elements => elements.map(el => el.textContent)
        );
        
        if (errorElements.length > 0) {
            console.log('❌ 发现错误组件:');
            errorElements.forEach(error => console.log(`  - ${error}`));
        }

        // 检查关键统计数据是否显示
        const statsData = await page.evaluate(() => {
            const stats = {};
            
            // 查找API统计
            const apiStats = document.querySelector('[data-testid="api-stats"], .ant-statistic');
            if (apiStats) {
                stats.apiVisible = true;
                stats.apiText = apiStats.textContent;
            }
            
            // 查找服务状态
            const serviceCards = document.querySelectorAll('.ant-card');
            stats.serviceCards = serviceCards.length;
            
            // 查找加载状态
            const loadingElements = document.querySelectorAll('.ant-spin, [class*="loading"]');
            stats.loadingElements = loadingElements.length;
            
            return stats;
        });

        console.log('📊 页面组件状态:');
        console.log(`  - 服务卡片数量: ${statsData.serviceCards}`);
        console.log(`  - 加载元素数量: ${statsData.loadingElements}`);
        console.log(`  - API统计可见: ${statsData.apiVisible || false}`);
        
        if (statsData.apiText) {
            console.log(`  - API统计文本: ${statsData.apiText.substring(0, 100)}...`);
        }

        // 截图保存
        await page.screenshot({
            path: '/home/ubuntu/arbitrage-frontend-v5.1/debug-screenshot.png',
            fullPage: true
        });
        console.log('📸 页面截图已保存: debug-screenshot.png');

        // 获取网络请求统计
        const performance = await page.evaluate(() => {
            return JSON.parse(JSON.stringify(performance.getEntries()
                .filter(entry => entry.name.includes('/api/'))
                .map(entry => ({
                    name: entry.name,
                    duration: entry.duration,
                    responseStart: entry.responseStart
                }))
            ));
        });

        console.log('🌐 API请求性能:');
        performance.forEach(req => {
            console.log(`  - ${req.name}: ${Math.round(req.duration)}ms`);
        });

    } catch (error) {
        console.error('❌ 调试过程发生错误:', error.message);
        
        if (error.message.includes('net::ERR_CONNECTION_REFUSED')) {
            console.log('🔧 前端服务可能未启动或端口错误');
        } else if (error.message.includes('Timeout')) {
            console.log('⏰ 页面加载超时，可能存在阻塞问题');
        }
    } finally {
        if (browser) {
            await browser.close();
        }
    }
}

debugFrontend().catch(console.error);