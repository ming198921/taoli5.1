// Qingxi 市场数据系统前端应用
class QingxiApp {
    constructor() {
        this.apiBaseUrl = 'http://localhost:50061/api/v1';
        this.refreshInterval = null;
        this.currentView = 'overview';
        this.exchanges = [];
        this.symbols = [];
        this.stats = {};
        
        this.init();
    }

    async init() {
        console.log('🚀 Qingxi App initializing...');
        
        // 初始化系统状态检查
        await this.checkSystemHealth();
        
        // 加载初始数据
        await this.loadInitialData();
        
        // 设置定时刷新
        this.startAutoRefresh();
        
        // 更新最后更新时间
        this.updateLastUpdateTime();
        
        console.log('✅ Qingxi App initialized successfully');
    }

    async checkSystemHealth() {
        try {
            showLoading();
            const response = await fetch(`${this.apiBaseUrl}/health`);
            const isHealthy = response.ok;
            
            const statusElement = document.getElementById('system-status');
            const statusLight = statusElement.querySelector('.status-light');
            const statusText = statusElement.querySelector('.status-text');
            
            if (isHealthy) {
                statusLight.className = 'status-light online';
                statusText.textContent = '系统正常';
                hideError();
            } else {
                statusLight.className = 'status-light offline';
                statusText.textContent = '系统异常';
                showError('系统健康检查失败');
            }
            
            hideLoading();
            return isHealthy;
        } catch (error) {
            console.error('Health check failed:', error);
            
            const statusElement = document.getElementById('system-status');
            const statusLight = statusElement.querySelector('.status-light');
            const statusText = statusElement.querySelector('.status-text');
            
            statusLight.className = 'status-light offline';
            statusText.textContent = '连接失败';
            
            showError('无法连接到API服务器');
            hideLoading();
            return false;
        }
    }

    async loadInitialData() {
        try {
            // 并行加载所有数据
            const [exchangesData, symbolsData, statsData] = await Promise.all([
                this.fetchExchanges(),
                this.fetchSymbols(),
                this.fetchStats()
            ]);
            
            this.exchanges = exchangesData || [];
            this.symbols = symbolsData || [];
            this.stats = statsData || {};
            
            // 更新统计卡片
            this.updateStatsCards();
            
            // 更新交易所过滤器
            this.updateExchangeFilter();
            
            showSuccess('数据加载完成');
        } catch (error) {
            console.error('Failed to load initial data:', error);
            showError('初始数据加载失败');
        }
    }

    async fetchExchanges() {
        try {
            const response = await fetch(`${this.apiBaseUrl}/exchanges`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            console.log('📊 Exchanges loaded:', data);
            return data.exchanges || [];
        } catch (error) {
            console.error('Failed to fetch exchanges:', error);
            return [];
        }
    }

    async fetchSymbols() {
        try {
            const response = await fetch(`${this.apiBaseUrl}/symbols`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            console.log('💰 Symbols loaded:', data);
            return data.symbols || [];
        } catch (error) {
            console.error('Failed to fetch symbols:', error);
            return [];
        }
    }

    async fetchStats() {
        try {
            const response = await fetch(`${this.apiBaseUrl}/stats`);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            console.log('📈 Stats loaded:', data);
            return data;
        } catch (error) {
            console.error('Failed to fetch stats:', error);
            return {};
        }
    }

    async fetchOrderbook(exchange = '', symbol = '') {
        try {
            let url = `${this.apiBaseUrl}/orderbook`;
            const params = new URLSearchParams();
            if (exchange) params.append('exchange', exchange);
            if (symbol) params.append('symbol', symbol);
            if (params.toString()) url += `?${params.toString()}`;
            
            const response = await fetch(url);
            if (!response.ok) throw new Error(`HTTP ${response.status}`);
            
            const data = await response.json();
            console.log('📋 Orderbook loaded:', data);
            return data;
        } catch (error) {
            console.error('Failed to fetch orderbook:', error);
            return null;
        }
    }

    updateStatsCards() {
        // 活跃交易所数量
        const activeExchanges = this.exchanges.filter(ex => ex.status === 'online').length;
        document.getElementById('active-exchanges').textContent = activeExchanges;
        
        // 总交易对数量
        document.getElementById('total-symbols').textContent = this.symbols.length;
        
        // 订单簿数量
        const orderbookCount = this.stats.orderbook_count || 0;
        document.getElementById('orderbook-count').textContent = orderbookCount;
        
        // 更新频率
        const updateRate = this.stats.updates_per_second || 0;
        document.getElementById('update-rate').textContent = updateRate.toFixed(1);
    }

    updateExchangeFilter() {
        const filterSelect = document.getElementById('exchange-filter');
        filterSelect.innerHTML = '<option value="">所有交易所</option>';
        
        this.exchanges.forEach(exchange => {
            const option = document.createElement('option');
            option.value = exchange.name;
            option.textContent = exchange.name.toUpperCase();
            filterSelect.appendChild(option);
        });
    }

    startAutoRefresh() {
        // 每30秒刷新一次数据
        this.refreshInterval = setInterval(async () => {
            await this.checkSystemHealth();
            await this.loadInitialData();
            this.updateLastUpdateTime();
        }, 30000);
    }

    updateLastUpdateTime() {
        const now = new Date();
        const timeString = now.toLocaleTimeString('zh-CN', { 
            hour12: false,
            hour: '2-digit',
            minute: '2-digit',
            second: '2-digit'
        });
        document.getElementById('last-update-time').textContent = timeString;
    }

    updateUptime() {
        // 模拟运行时间（在实际应用中应该从API获取）
        const startTime = new Date('2024-01-01T00:00:00');
        const now = new Date();
        const uptimeMs = now - startTime;
        
        const days = Math.floor(uptimeMs / (1000 * 60 * 60 * 24));
        const hours = Math.floor((uptimeMs % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
        const minutes = Math.floor((uptimeMs % (1000 * 60 * 60)) / (1000 * 60));
        
        const uptimeString = `${days}天 ${hours}小时 ${minutes}分钟`;
        document.getElementById('uptime').textContent = uptimeString;
    }

    // 视图切换方法
    showOverview() {
        this.currentView = 'overview';
        const content = document.getElementById('display-content');
        const title = document.getElementById('display-title');
        
        title.textContent = '系统概览';
        content.innerHTML = `
            <div class="welcome-message">
                <i class="fas fa-rocket"></i>
                <h3>Qingxi 市场数据系统</h3>
                <p>高性能多源加密货币市场数据采集与一致性验证系统</p>
                <div style="margin-top: 2rem;">
                    <div class="overview-metrics">
                        <div class="overview-metric">
                            <h4>支持交易所</h4>
                            <p>Binance, OKX, Huobi, Bybit, Gate.io</p>
                        </div>
                        <div class="overview-metric">
                            <h4>数据类型</h4>
                            <p>实时订单簿、交易数据、市场快照</p>
                        </div>
                        <div class="overview-metric">
                            <h4>特色功能</h4>
                            <p>跨交易所一致性验证、异常检测、动态配置</p>
                        </div>
                    </div>
                </div>
            </div>
        `;
        content.className = 'display-content fade-in';
    }

    showExchanges() {
        this.currentView = 'exchanges';
        const content = document.getElementById('display-content');
        const title = document.getElementById('display-title');
        
        title.textContent = '交易所状态监控';
        
        if (this.exchanges.length === 0) {
            content.innerHTML = `
                <div class="welcome-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <h3>暂无交易所数据</h3>
                    <p>请检查API连接或稍后重试</p>
                </div>
            `;
            return;
        }
        
        const exchangeCards = this.exchanges.map(exchange => `
            <div class="exchange-card">
                <div class="exchange-header">
                    <div class="exchange-name">${exchange.name.toUpperCase()}</div>
                    <div class="exchange-status ${exchange.status === 'online' ? 'status-online' : 'status-offline'}">
                        ${exchange.status === 'online' ? '在线' : '离线'}
                    </div>
                </div>
                <div class="exchange-info">
                    <p><strong>WebSocket:</strong> ${exchange.websocket_url || 'N/A'}</p>
                    <p><strong>REST API:</strong> ${exchange.rest_api_url || 'N/A'}</p>
                    <p><strong>支持交易对:</strong> ${exchange.supported_symbols || 0} 个</p>
                </div>
                <div class="exchange-metrics">
                    <div class="metric">
                        <div class="metric-value">${exchange.latency || '--'}</div>
                        <div class="metric-label">延迟 (ms)</div>
                    </div>
                    <div class="metric">
                        <div class="metric-value">${exchange.uptime || '--'}</div>
                        <div class="metric-label">正常运行率</div>
                    </div>
                </div>
            </div>
        `).join('');
        
        content.innerHTML = `<div class="exchange-grid">${exchangeCards}</div>`;
        content.className = 'display-content fade-in';
    }

    showSymbols() {
        this.currentView = 'symbols';
        const content = document.getElementById('display-content');
        const title = document.getElementById('display-title');
        
        title.textContent = '交易对管理';
        
        if (this.symbols.length === 0) {
            content.innerHTML = `
                <div class="welcome-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <h3>暂无交易对数据</h3>
                    <p>请检查API连接或稍后重试</p>
                </div>
            `;
            return;
        }
        
        const symbolRows = this.symbols.map(symbol => `
            <tr>
                <td><strong>${symbol.base}/${symbol.quote}</strong></td>
                <td>${symbol.exchanges ? symbol.exchanges.join(', ') : 'N/A'}</td>
                <td><span class="status-indicator ${symbol.status === 'active' ? 'online' : 'offline'}">${symbol.status === 'active' ? '活跃' : '暂停'}</span></td>
                <td>${symbol.last_price || '--'}</td>
                <td>${symbol.volume_24h || '--'}</td>
                <td>${symbol.last_update || '--'}</td>
            </tr>
        `).join('');
        
        content.innerHTML = `
            <table class="data-table">
                <thead>
                    <tr>
                        <th>交易对</th>
                        <th>支持交易所</th>
                        <th>状态</th>
                        <th>最新价格</th>
                        <th>24h成交量</th>
                        <th>最后更新</th>
                    </tr>
                </thead>
                <tbody>
                    ${symbolRows}
                </tbody>
            </table>
        `;
        content.className = 'display-content fade-in';
    }

    showStats() {
        this.currentView = 'stats';
        const content = document.getElementById('display-content');
        const title = document.getElementById('display-title');
        
        title.textContent = '系统统计信息';
        
        const statsData = [
            { label: '活跃订单簿数量', value: this.stats.orderbook_count || 0, unit: '个' },
            { label: '批处理项目数', value: this.stats.batch_processed_count || 0, unit: '个' },
            { label: '更新频率', value: (this.stats.updates_per_second || 0).toFixed(2), unit: '次/秒' },
            { label: '平均延迟', value: (this.stats.average_latency || 0).toFixed(2), unit: 'ms' },
            { label: '数据完整性', value: ((this.stats.data_integrity || 0) * 100).toFixed(1), unit: '%' },
            { label: '系统负载', value: ((this.stats.system_load || 0) * 100).toFixed(1), unit: '%' }
        ];
        
        const statsRows = statsData.map(stat => `
            <tr>
                <td><strong>${stat.label}</strong></td>
                <td>${stat.value} ${stat.unit}</td>
                <td><div class="metric-bar"><div class="metric-fill" style="width: ${Math.min(100, stat.value)}%"></div></div></td>
            </tr>
        `).join('');
        
        content.innerHTML = `
            <div class="stats-overview">
                <p>系统性能指标和统计信息概览</p>
            </div>
            <table class="data-table">
                <thead>
                    <tr>
                        <th>指标</th>
                        <th>当前值</th>
                        <th>可视化</th>
                    </tr>
                </thead>
                <tbody>
                    ${statsRows}
                </tbody>
            </table>
        `;
        content.className = 'display-content fade-in';
    }

    async showOrderbook() {
        this.currentView = 'orderbook';
        const content = document.getElementById('display-content');
        const title = document.getElementById('display-title');
        
        title.textContent = '实时订单簿';
        
        showLoading();
        const orderbookData = await this.fetchOrderbook();
        hideLoading();
        
        if (!orderbookData || !orderbookData.bids || !orderbookData.asks) {
            content.innerHTML = `
                <div class="welcome-message">
                    <i class="fas fa-exclamation-triangle"></i>
                    <h3>暂无订单簿数据</h3>
                    <p>请检查API连接或稍后重试</p>
                </div>
            `;
            return;
        }
        
        const bidsRows = orderbookData.bids.slice(0, 10).map((bid, index) => `
            <tr>
                <td>${index + 1}</td>
                <td class="price-bid">${bid.price}</td>
                <td>${bid.quantity}</td>
                <td>${(bid.price * bid.quantity).toFixed(2)}</td>
            </tr>
        `).join('');
        
        const asksRows = orderbookData.asks.slice(0, 10).map((ask, index) => `
            <tr>
                <td>${index + 1}</td>
                <td class="price-ask">${ask.price}</td>
                <td>${ask.quantity}</td>
                <td>${(ask.price * ask.quantity).toFixed(2)}</td>
            </tr>
        `).join('');
        
        content.innerHTML = `
            <div class="orderbook-container">
                <div class="orderbook-info">
                    <p><strong>交易对:</strong> ${orderbookData.symbol || 'BTC/USDT'}</p>
                    <p><strong>交易所:</strong> ${orderbookData.exchange || 'Multiple'}</p>
                    <p><strong>更新时间:</strong> ${orderbookData.timestamp || new Date().toISOString()}</p>
                </div>
                
                <div class="orderbook-tables">
                    <div class="orderbook-side">
                        <h4>买单 (Bids)</h4>
                        <table class="data-table orderbook-table">
                            <thead>
                                <tr>
                                    <th>#</th>
                                    <th>价格</th>
                                    <th>数量</th>
                                    <th>总值</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${bidsRows}
                            </tbody>
                        </table>
                    </div>
                    
                    <div class="orderbook-side">
                        <h4>卖单 (Asks)</h4>
                        <table class="data-table orderbook-table">
                            <thead>
                                <tr>
                                    <th>#</th>
                                    <th>价格</th>
                                    <th>数量</th>
                                    <th>总值</th>
                                </tr>
                            </thead>
                            <tbody>
                                ${asksRows}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
        `;
        content.className = 'display-content fade-in';
    }

    filterData() {
        const selectedExchange = document.getElementById('exchange-filter').value;
        console.log('Filtering by exchange:', selectedExchange);
        
        // 根据当前视图重新加载数据
        switch (this.currentView) {
            case 'exchanges':
                this.showExchanges();
                break;
            case 'symbols':
                this.showSymbols();
                break;
            case 'orderbook':
                this.showOrderbook();
                break;
            default:
                this.showOverview();
        }
    }

    exportData() {
        const data = {
            timestamp: new Date().toISOString(),
            exchanges: this.exchanges,
            symbols: this.symbols,
            stats: this.stats
        };
        
        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `qingxi-data-${new Date().toISOString().split('T')[0]}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        showSuccess('数据导出成功');
    }

    async refreshAllData() {
        console.log('🔄 Refreshing all data...');
        showLoading();
        
        try {
            await this.checkSystemHealth();
            await this.loadInitialData();
            this.updateLastUpdateTime();
            
            // 刷新当前视图
            switch (this.currentView) {
                case 'exchanges':
                    this.showExchanges();
                    break;
                case 'symbols':
                    this.showSymbols();
                    break;
                case 'stats':
                    this.showStats();
                    break;
                case 'orderbook':
                    await this.showOrderbook();
                    break;
                default:
                    this.showOverview();
            }
            
            showSuccess('数据刷新完成');
        } catch (error) {
            console.error('Failed to refresh data:', error);
            showError('数据刷新失败');
        } finally {
            hideLoading();
        }
    }

    destroy() {
        if (this.refreshInterval) {
            clearInterval(this.refreshInterval);
        }
    }
}

// 全局实用函数
function showLoading() {
    document.getElementById('loading-overlay').style.display = 'flex';
}

function hideLoading() {
    document.getElementById('loading-overlay').style.display = 'none';
}

function showError(message) {
    const errorToast = document.getElementById('error-toast');
    const errorMessage = document.getElementById('error-message');
    errorMessage.textContent = message;
    errorToast.style.display = 'flex';
    
    // 5秒后自动隐藏
    setTimeout(hideError, 5000);
}

function hideError() {
    document.getElementById('error-toast').style.display = 'none';
}

function showSuccess(message) {
    const successToast = document.getElementById('success-toast');
    const successMessage = document.getElementById('success-message');
    successMessage.textContent = message;
    successToast.style.display = 'flex';
    
    // 3秒后自动隐藏
    setTimeout(hideSuccess, 3000);
}

function hideSuccess() {
    document.getElementById('success-toast').style.display = 'none';
}

// 全局函数，供HTML调用
function showExchanges() {
    window.qingxiApp.showExchanges();
}

function showSymbols() {
    window.qingxiApp.showSymbols();
}

function showStats() {
    window.qingxiApp.showStats();
}

function showOrderbook() {
    window.qingxiApp.showOrderbook();
}

function refreshAllData() {
    window.qingxiApp.refreshAllData();
}

function filterData() {
    window.qingxiApp.filterData();
}

function exportData() {
    window.qingxiApp.exportData();
}

// 应用启动
document.addEventListener('DOMContentLoaded', () => {
    window.qingxiApp = new QingxiApp();
    
    // 定期更新运行时间
    setInterval(() => {
        window.qingxiApp.updateUptime();
    }, 60000); // 每分钟更新一次
});

// 页面卸载时清理
window.addEventListener('beforeunload', () => {
    if (window.qingxiApp) {
        window.qingxiApp.destroy();
    }
});
