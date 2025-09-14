/**
 * 5.1套利系统前端SDK集成
 * 封装我们完整的API网关SDK，提供统一的前端控制接口
 */

import axios from 'axios';

// API基础配置
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3001';

// 创建HTTP客户端
class HttpClient {
  constructor(config = {}) {
    this.config = {
      baseURL: API_BASE_URL,
      timeout: 30000,
      enableLogging: true,
      retryAttempts: 3,
      retryDelay: 1000,
      ...config,
    };

    this.client = axios.create({
      baseURL: this.config.baseURL,
      timeout: this.config.timeout,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
    });

    this.authToken = null;
    this.setupInterceptors();
  }

  setupInterceptors() {
    // 请求拦截器
    this.client.interceptors.request.use(
      (config) => {
        // 添加认证令牌
        if (this.authToken) {
          config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // 添加请求ID
        config.headers['X-Request-ID'] = this.generateRequestId();

        if (this.config.enableLogging) {
          console.log(`[HTTP] ${config.method?.toUpperCase()} ${config.url}`);
        }

        return config;
      },
      (error) => {
        if (this.config.enableLogging) {
          console.error('[HTTP] Request Error:', error);
        }
        return Promise.reject(error);
      }
    );

    // 响应拦截器
    this.client.interceptors.response.use(
      (response) => {
        if (this.config.enableLogging) {
          console.log(`[HTTP] Response ${response.status}: ${response.config.url}`);
        }
        return response;
      },
      async (error) => {
        if (this.config.enableLogging) {
          console.error('[HTTP] Response Error:', error.response?.status, error.response?.data);
        }

        // 401错误 - 令牌过期，尝试刷新
        if (error.response?.status === 401) {
          this.clearAuthToken();
          // 可以在这里触发重新登录
          window.dispatchEvent(new CustomEvent('auth:required'));
        }

        // 网络错误
        if (!error.response) {
          const networkError = new Error('网络连接失败，请检查网络或后端服务状态');
          networkError.isNetworkError = true;
          throw networkError;
        }

        // API错误
        const apiError = error.response.data;
        const serverError = new Error(apiError?.message || `服务器错误 (${error.response.status})`);
        serverError.isServerError = true;
        serverError.status = error.response.status;
        throw serverError;
      }
    );
  }

  setAuthToken(token) {
    this.authToken = token;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('arbitrage_auth_token', token);
    }
  }

  clearAuthToken() {
    this.authToken = null;
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem('arbitrage_auth_token');
      localStorage.removeItem('arbitrage_refresh_token');
    }
  }

  getStoredAuthToken() {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('arbitrage_auth_token');
    }
    return null;
  }

  generateRequestId() {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  async get(url, config) {
    const response = await this.client.get(url, config);
    return response.data;
  }

  async post(url, data, config) {
    const response = await this.client.post(url, data, config);
    return response.data;
  }

  async put(url, data, config) {
    const response = await this.client.put(url, data, config);
    return response.data;
  }

  async delete(url, config) {
    const response = await this.client.delete(url, config);
    return response.data;
  }
}

// 认证服务
class AuthService {
  constructor(httpClient) {
    this.httpClient = httpClient;
  }

  async login({ username, password, remember = false }) {
    const response = await this.httpClient.post('/api/auth/login', {
      username,
      password,
      remember,
    });

    if (response.success && response.data) {
      this.httpClient.setAuthToken(response.data.token);
      if (response.data.refresh_token) {
        localStorage.setItem('arbitrage_refresh_token', response.data.refresh_token);
      }
      return response.data;
    }

    throw new Error(response.message || '登录失败');
  }

  async logout() {
    try {
      await this.httpClient.post('/api/auth/logout');
    } finally {
      this.httpClient.clearAuthToken();
    }
  }

  async getCurrentUser() {
    const response = await this.httpClient.get('/api/auth/me');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取用户信息失败');
  }

  async hasPermission(permission) {
    try {
      const response = await this.httpClient.get('/api/auth/permissions');
      if (response.success && response.data) {
        return response.data.includes(permission) || response.data.includes('*');
      }
      return false;
    } catch {
      return false;
    }
  }
}

// QingXi数据服务
class QingxiService {
  constructor(httpClient) {
    this.httpClient = httpClient;
  }

  async getMarketData(symbol, exchange) {
    const params = {};
    if (symbol) params.symbol = symbol;
    if (exchange) params.exchange = exchange;

    const response = await this.httpClient.get('/api/qingxi/market-data', { params });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取市场数据失败');
  }

  async getCollectorStatus() {
    const response = await this.httpClient.get('/api/qingxi/collectors');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取收集器状态失败');
  }

  async startCollector(collectorId) {
    const response = await this.httpClient.post(`/api/qingxi/collectors/${collectorId}/start`);
    if (!response.success) {
      throw new Error(response.message || '启动收集器失败');
    }
  }

  async stopCollector(collectorId) {
    const response = await this.httpClient.post(`/api/qingxi/collectors/${collectorId}/stop`);
    if (!response.success) {
      throw new Error(response.message || '停止收集器失败');
    }
  }

  async getArbitrageOpportunities(query = {}) {
    const response = await this.httpClient.get('/api/qingxi/arbitrage-opportunities', { params: query });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取套利机会失败');
  }

  async getDataStats() {
    const response = await this.httpClient.get('/api/qingxi/stats');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取数据统计失败');
  }

  async getDataQuality() {
    const response = await this.httpClient.get('/api/qingxi/data/quality');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取数据质量失败');
  }

  async getDataFlow() {
    const response = await this.httpClient.get('/api/qingxi/data/flow');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取数据流量失败');
  }

  async getCleanStats() {
    const response = await this.httpClient.get('/api/qingxi/clean-stats');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取清洗速度统计失败');
  }
}

// 仪表板服务
class DashboardService {
  constructor(httpClient) {
    this.httpClient = httpClient;
  }

  async getDashboardStats(timeRange) {
    const response = await this.httpClient.get('/api/dashboard/stats', { params: timeRange });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取仪表板统计失败');
  }

  async getSankeyData(params) {
    const response = await this.httpClient.get('/api/dashboard/sankey', { params });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取Sankey数据失败');
  }

  async getProfitCurve(params) {
    const response = await this.httpClient.get('/api/dashboard/profit-curve', { params });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取利润曲线失败');
  }

  async getFlowHistory(query) {
    const response = await this.httpClient.get('/api/dashboard/flow-history', { params: query });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取流动历史失败');
  }

  async getRealTimeFlows() {
    const response = await this.httpClient.get('/api/dashboard/realtime-flows');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取实时交易流失败');
  }
}

// 监控服务
class MonitoringService {
  constructor(httpClient) {
    this.httpClient = httpClient;
  }

  async getHealthChecks() {
    const response = await this.httpClient.get('/api/monitoring/health');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取健康检查失败');
  }

  async getSystemMetrics(timeRange, granularity) {
    const response = await this.httpClient.get('/api/monitoring/metrics', {
      params: { ...timeRange, granularity }
    });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取系统指标失败');
  }

  async getCurrentMetrics() {
    const response = await this.httpClient.get('/api/monitoring/metrics/current');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取当前系统指标失败');
  }

  async getAlerts(query) {
    const response = await this.httpClient.get('/api/monitoring/alerts', { params: query });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取警报列表失败');
  }

  async acknowledgeAlert(alertId, note) {
    const response = await this.httpClient.post(`/api/monitoring/alerts/${alertId}/acknowledge`, { note });
    if (!response.success) {
      throw new Error(response.message || '确认警报失败');
    }
  }

  async getAlertRules(query) {
    const response = await this.httpClient.get('/api/monitoring/alert-rules', { params: query });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取警报规则失败');
  }

  async createAlertRule(rule) {
    const response = await this.httpClient.post('/api/monitoring/alert-rules', rule);
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '创建警报规则失败');
  }
}

// 系统服务
class SystemService {
  constructor(httpClient) {
    this.httpClient = httpClient;
  }

  async getSystemStatus() {
    try {
      const response = await this.httpClient.get('/api/system/status');
      console.log('GetSystemStatus response:', response);
      if (response && response.success && response.data) {
        return response.data;
      }
      throw new Error(response?.message || '获取系统状态失败');
    } catch (error) {
      console.error('GetSystemStatus error:', error);
      throw new Error(error.message || '网络连接失败');
    }
  }

  async startSystem() {
    try {
      const response = await this.httpClient.post('/api/system/start');
      console.log('StartSystem response:', response);
      if (response && response.success) {
        return response;
      }
      throw new Error(response?.message || '启动系统失败');
    } catch (error) {
      console.error('StartSystem error:', error);
      throw new Error(error.message || '网络连接失败');
    }
  }

  async stopSystem() {
    try {
      const response = await this.httpClient.post('/api/system/stop');
      console.log('StopSystem response:', response);
      if (response && response.success) {
        return response;
      }
      throw new Error(response?.message || '停止系统失败');
    } catch (error) {
      console.error('StopSystem error:', error);
      throw new Error(error.message || '网络连接失败');
    }
  }

  async healthCheck() {
    try {
      const response = await this.httpClient.get('/api/system/status');
      return response;
    } catch (error) {
      throw new Error('健康检查失败');
    }
  }

  async getSystemLogs(params) {
    const response = await this.httpClient.get('/api/system/logs', { params });
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取系统日志失败');
  }

  async getSystemStats() {
    const response = await this.httpClient.get('/api/system/stats');
    if (response.success && response.data) {
      return response.data;
    }
    throw new Error(response.message || '获取系统统计失败');
  }
}

// WebSocket客户端
class WebSocketClient {
  constructor(config = {}) {
    this.config = {
      wsUrl: config.wsUrl || API_BASE_URL.replace(/^http/, 'ws') + '/ws',
      enableLogging: config.enableLogging || true,
      ...config,
    };
    
    this.ws = null;
    this.isConnected = false;
    this.isConnecting = false;
    this.authToken = null;
    this.eventListeners = new Map();
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 5;
  }

  async connect(authToken) {
    if (this.isConnected || this.isConnecting) {
      return;
    }

    this.isConnecting = true;
    this.authToken = authToken;

    return new Promise((resolve, reject) => {
      try {
        const wsUrl = this.buildWsUrl();
        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
          this.isConnected = true;
          this.isConnecting = false;
          this.reconnectAttempts = 0;

          if (this.config.enableLogging) {
            console.log('[WebSocket] Connected successfully');
          }

          this.emit('connected', {});
          resolve();
        };

        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        this.ws.onclose = (event) => {
          this.isConnected = false;
          this.isConnecting = false;

          if (this.config.enableLogging) {
            console.log('[WebSocket] Connection closed:', event.code, event.reason);
          }

          this.emit('disconnected', { code: event.code, reason: event.reason });

          if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
            this.scheduleReconnect();
          }
        };

        this.ws.onerror = (error) => {
          if (this.config.enableLogging) {
            console.error('[WebSocket] Connection error:', error);
          }

          this.emit('error', error);

          if (this.isConnecting) {
            this.isConnecting = false;
            reject(new Error('WebSocket连接失败'));
          }
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  disconnect() {
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
    }
    this.isConnected = false;
    this.isConnecting = false;
  }

  subscribe(eventType, callback) {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }

    this.eventListeners.get(eventType).add(callback);

    if (this.isConnected) {
      this.send({
        type: 'subscribe',
        event: eventType,
      });
    }

    return {
      unsubscribe: () => {
        const listeners = this.eventListeners.get(eventType);
        if (listeners) {
          listeners.delete(callback);
          if (listeners.size === 0) {
            this.eventListeners.delete(eventType);
          }
        }
      }
    };
  }

  send(message) {
    if (!this.isConnected || !this.ws) {
      throw new Error('WebSocket未连接');
    }

    const payload = {
      ...message,
      timestamp: Date.now(),
    };

    this.ws.send(JSON.stringify(payload));

    if (this.config.enableLogging) {
      console.log('[WebSocket] Sent message:', payload);
    }
  }

  handleMessage(data) {
    try {
      const message = JSON.parse(data);

      if (this.config.enableLogging) {
        console.log('[WebSocket] Received message:', message);
      }

      this.emit(message.type, message.data);
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[WebSocket] Failed to parse message:', data, error);
      }
    }
  }

  emit(eventType, data) {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          if (this.config.enableLogging) {
            console.error(`[WebSocket] Error in ${eventType} callback:`, error);
          }
        }
      });
    }
  }

  buildWsUrl() {
    const url = new URL(this.config.wsUrl);
    if (this.authToken) {
      url.searchParams.set('token', this.authToken);
    }
    return url.toString();
  }

  scheduleReconnect() {
    this.reconnectAttempts++;
    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 30000);

    if (this.config.enableLogging) {
      console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
    }

    setTimeout(() => {
      if (!this.isConnected) {
        this.connect(this.authToken).catch(error => {
          if (this.config.enableLogging) {
            console.error('[WebSocket] Reconnect failed:', error);
          }
        });
      }
    }, delay);
  }

  getConnectionStatus() {
    return {
      connected: this.isConnected,
      connecting: this.isConnecting,
      reconnectAttempts: this.reconnectAttempts,
    };
  }
}

// 主SDK类
class ArbitrageSystemSDK {
  constructor(config = {}) {
    this.config = {
      baseUrl: API_BASE_URL,
      wsUrl: config.wsUrl || API_BASE_URL.replace(/^http/, 'ws') + '/ws',
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      enableLogging: true,
      ...config,
    };

    // 初始化核心客户端
    this.httpClient = new HttpClient(this.config);
    this.wsClient = new WebSocketClient(this.config);

    // 初始化服务
    this.auth = new AuthService(this.httpClient);
    this.qingxi = new QingxiService(this.httpClient);
    this.dashboard = new DashboardService(this.httpClient);
    this.monitoring = new MonitoringService(this.httpClient);
    this.system = new SystemService(this.httpClient);

    this.isInitialized = false;
    this.currentUser = null;

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] Initialized with config:', {
        baseUrl: this.config.baseUrl,
        wsUrl: this.config.wsUrl,
        timeout: this.config.timeout,
      });
    }
  }

  async initialize() {
    if (this.isInitialized) {
      return;
    }

    try {
      // 尝试自动登录
      const token = this.httpClient.getStoredAuthToken();
      if (token) {
        this.httpClient.setAuthToken(token);
        try {
          this.currentUser = await this.auth.getCurrentUser();
          if (this.config.enableLogging) {
            console.log('[ArbitrageSDK] Auto-login successful:', this.currentUser.username);
          }
        } catch (error) {
          if (this.config.enableLogging) {
            console.warn('[ArbitrageSDK] Auto-login failed:', error);
          }
          this.httpClient.clearAuthToken();
        }
      }

      this.isInitialized = true;

      if (this.config.enableLogging) {
        console.log('[ArbitrageSDK] Initialization completed');
      }
    } catch (error) {
      if (this.config.enableLogging) {
        console.warn('[ArbitrageSDK] Initialization failed:', error);
      }
      this.isInitialized = true;
    }
  }

  async login(credentials) {
    try {
      const response = await this.auth.login(credentials);
      this.currentUser = response.user;

      if (this.config.wsUrl) {
        await this.connectWebSocket(response.token);
      }

      if (this.config.enableLogging) {
        console.log('[ArbitrageSDK] Login successful:', this.currentUser.username);
      }

      return this.currentUser;
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[ArbitrageSDK] Login failed:', error);
      }
      throw error;
    }
  }

  async logout() {
    try {
      await this.auth.logout();
      this.currentUser = null;
      this.disconnectWebSocket();

      if (this.config.enableLogging) {
        console.log('[ArbitrageSDK] Logout successful');
      }
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[ArbitrageSDK] Logout error:', error);
      }
      this.currentUser = null;
      this.disconnectWebSocket();
    }
  }

  async connectWebSocket(token) {
    const authToken = token || this.httpClient.getStoredAuthToken();
    if (!authToken) {
      throw new Error('未找到认证令牌');
    }

    await this.wsClient.connect(authToken);

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] WebSocket connected');
    }
  }

  disconnectWebSocket() {
    this.wsClient.disconnect();

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] WebSocket disconnected');
    }
  }

  subscribe(eventType, callback) {
    return this.wsClient.subscribe(eventType, callback);
  }

  subscribeMarketData(callback) {
    return this.subscribe('market_data', callback);
  }

  subscribeArbitrageOpportunities(callback) {
    return this.subscribe('arbitrage_opportunity', callback);
  }

  subscribeAlerts(callback) {
    return this.subscribe('alert', callback);
  }

  subscribeSystemStatus(callback) {
    return this.subscribe('system_status', callback);
  }

  getCurrentUser() {
    return this.currentUser;
  }

  isLoggedIn() {
    return !!this.currentUser && !!this.httpClient.getStoredAuthToken();
  }

  getWebSocketStatus() {
    return this.wsClient.getConnectionStatus();
  }

  async healthCheck() {
    const health = {
      sdk: this.isInitialized,
      api: false,
      websocket: false,
      user: this.isLoggedIn(),
    };

    try {
      await this.system.healthCheck();
      health.api = true;
    } catch {
      health.api = false;
    }

    health.websocket = this.wsClient.getConnectionStatus().connected;

    return health;
  }

  getStatus() {
    const wsStatus = this.wsClient.getConnectionStatus();
    
    return {
      initialized: this.isInitialized,
      loggedIn: this.isLoggedIn(),
      user: this.currentUser,
      httpConnected: true,
      wsConnected: wsStatus.connected,
      wsConnecting: wsStatus.connecting,
    };
  }

  static formatError(error) {
    return {
      message: error?.message || '未知错误',
      details: error,
    };
  }
}

// 创建全局SDK实例
const WS_BASE_URL = API_BASE_URL.replace(/^http/, 'ws');
export const arbitrageSDK = new ArbitrageSystemSDK({
  baseUrl: API_BASE_URL,
  wsUrl: `${WS_BASE_URL}/ws`,
  enableLogging: true,
});

// 自动初始化
arbitrageSDK.initialize().catch(console.error);

// 导出SDK类和实例
export { ArbitrageSystemSDK };
export default arbitrageSDK;