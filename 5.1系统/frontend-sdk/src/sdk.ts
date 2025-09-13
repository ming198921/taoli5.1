/**
 * 5.1套利系统前端SDK主入口
 * 提供完整的API访问和实时数据功能，实现前端100%控制后端
 */

import { HttpClient } from './core/http-client';
import { WebSocketClient } from './core/websocket-client';
import { AuthService } from './services/auth.service';
import { QingxiService } from './services/qingxi.service';
import { DashboardService } from './services/dashboard.service';
import { MonitoringService } from './services/monitoring.service';
import { SystemService } from './services/system.service';
import { 
  SDKConfig, 
  ArbitrageSDKError, 
  EventCallback, 
  EventSubscription,
  UserInfo,
  LoginRequest,
} from './types';

export class ArbitrageSystemSDK {
  private httpClient: HttpClient;
  private wsClient: WebSocketClient;
  
  // 服务实例
  public auth: AuthService;
  public qingxi: QingxiService;
  public dashboard: DashboardService;
  public monitoring: MonitoringService;
  public system: SystemService;
  
  // 配置和状态
  private config: SDKConfig;
  private isInitialized: boolean = false;
  private currentUser?: UserInfo;

  constructor(config: SDKConfig) {
    this.config = {
      timeout: 30000,
      retryAttempts: 3,
      retryDelay: 1000,
      enableLogging: false,
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

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] Initialized with config:', {
        baseUrl: this.config.baseUrl,
        wsUrl: this.config.wsUrl,
        timeout: this.config.timeout,
      });
    }
  }

  /**
   * 初始化SDK
   */
  public async initialize(): Promise<void> {
    if (this.isInitialized) {
      return;
    }

    try {
      // 尝试自动登录
      const user = await this.auth.autoLogin();
      if (user) {
        this.currentUser = user;
        if (this.config.enableLogging) {
          console.log('[ArbitrageSDK] Auto-login successful:', user.username);
        }
      }

      this.isInitialized = true;

      if (this.config.enableLogging) {
        console.log('[ArbitrageSDK] Initialization completed');
      }
    } catch (error) {
      if (this.config.enableLogging) {
        console.warn('[ArbitrageSDK] Auto-login failed:', error);
      }
      // 自动登录失败不阻塞初始化
      this.isInitialized = true;
    }
  }

  /**
   * 用户登录
   */
  public async login(credentials: LoginRequest): Promise<UserInfo> {
    try {
      const response = await this.auth.login(credentials);
      this.currentUser = response.user;

      // 连接WebSocket（如果需要）
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

  /**
   * 用户登出
   */
  public async logout(): Promise<void> {
    try {
      await this.auth.logout();
      this.currentUser = undefined;
      this.disconnectWebSocket();

      if (this.config.enableLogging) {
        console.log('[ArbitrageSDK] Logout successful');
      }
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[ArbitrageSDK] Logout error:', error);
      }
      // 即使登出失败也清除本地状态
      this.currentUser = undefined;
      this.disconnectWebSocket();
    }
  }

  /**
   * 连接WebSocket
   */
  public async connectWebSocket(token?: string): Promise<void> {
    if (!this.config.wsUrl) {
      throw new ArbitrageSDKError('WebSocket URL未配置', 'WS_URL_MISSING');
    }

    const authToken = token || this.httpClient.getStoredAuthToken();
    if (!authToken) {
      throw new ArbitrageSDKError('未找到认证令牌', 'AUTH_TOKEN_MISSING');
    }

    await this.wsClient.connect(authToken);

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] WebSocket connected');
    }
  }

  /**
   * 断开WebSocket连接
   */
  public disconnectWebSocket(): void {
    this.wsClient.disconnect();

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] WebSocket disconnected');
    }
  }

  /**
   * 获取WebSocket连接状态
   */
  public getWebSocketStatus(): {
    connected: boolean;
    connecting: boolean;
    reconnectAttempts: number;
  } {
    return this.wsClient.getConnectionStatus();
  }

  /**
   * 订阅WebSocket事件
   */
  public subscribe<T = any>(
    eventType: string,
    callback: EventCallback<T>
  ): EventSubscription {
    return this.wsClient.subscribe(eventType, callback);
  }

  /**
   * 订阅市场数据更新
   */
  public subscribeMarketData(callback: EventCallback): EventSubscription {
    return this.wsClient.subscribeMarketData(callback);
  }

  /**
   * 订阅订单簿更新
   */
  public subscribeOrderBook(callback: EventCallback): EventSubscription {
    return this.wsClient.subscribeOrderBook(callback);
  }

  /**
   * 订阅套利机会
   */
  public subscribeArbitrageOpportunities(callback: EventCallback): EventSubscription {
    return this.wsClient.subscribeArbitrageOpportunities(callback);
  }

  /**
   * 订阅系统警报
   */
  public subscribeAlerts(callback: EventCallback): EventSubscription {
    return this.wsClient.subscribeAlerts(callback);
  }

  /**
   * 订阅系统状态更新
   */
  public subscribeSystemStatus(callback: EventCallback): EventSubscription {
    return this.wsClient.subscribeSystemStatus(callback);
  }

  /**
   * 获取当前用户信息
   */
  public getCurrentUser(): UserInfo | undefined {
    return this.currentUser;
  }

  /**
   * 检查是否已登录
   */
  public isLoggedIn(): boolean {
    return !!this.currentUser && !!this.httpClient.getStoredAuthToken();
  }

  /**
   * 检查用户权限
   */
  public async hasPermission(permission: string): Promise<boolean> {
    if (!this.isLoggedIn()) {
      return false;
    }
    return this.auth.hasPermission(permission);
  }

  /**
   * 检查是否为管理员
   */
  public async isAdmin(): Promise<boolean> {
    if (!this.isLoggedIn()) {
      return false;
    }
    return this.auth.isAdmin();
  }

  /**
   * 获取SDK配置
   */
  public getConfig(): SDKConfig {
    return { ...this.config };
  }

  /**
   * 更新SDK配置
   */
  public updateConfig(updates: Partial<SDKConfig>): void {
    this.config = { ...this.config, ...updates };
    
    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] Configuration updated:', updates);
    }
  }

  /**
   * 获取SDK状态
   */
  public getStatus(): {
    initialized: boolean;
    loggedIn: boolean;
    user?: UserInfo;
    httpConnected: boolean;
    wsConnected: boolean;
    wsConnecting: boolean;
  } {
    const wsStatus = this.wsClient.getConnectionStatus();
    
    return {
      initialized: this.isInitialized,
      loggedIn: this.isLoggedIn(),
      user: this.currentUser,
      httpConnected: true, // HTTP连接总是可用的
      wsConnected: wsStatus.connected,
      wsConnecting: wsStatus.connecting,
    };
  }

  /**
   * 执行健康检查
   */
  public async healthCheck(): Promise<{
    sdk: boolean;
    api: boolean;
    websocket: boolean;
    user: boolean;
  }> {
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

  /**
   * 刷新用户信息
   */
  public async refreshUserInfo(): Promise<UserInfo | null> {
    if (!this.isLoggedIn()) {
      return null;
    }

    try {
      this.currentUser = await this.auth.getCurrentUser();
      return this.currentUser;
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[ArbitrageSDK] Failed to refresh user info:', error);
      }
      return null;
    }
  }

  /**
   * 销毁SDK实例
   */
  public destroy(): void {
    this.disconnectWebSocket();
    this.currentUser = undefined;
    this.isInitialized = false;

    if (this.config.enableLogging) {
      console.log('[ArbitrageSDK] SDK instance destroyed');
    }
  }

  /**
   * 创建新的SDK实例（静态方法）
   */
  public static create(config: SDKConfig): ArbitrageSystemSDK {
    return new ArbitrageSystemSDK(config);
  }

  /**
   * 批量操作助手
   */
  public batch() {
    const operations: Array<() => Promise<any>> = [];

    return {
      /**
       * 添加操作到批次
       */
      add<T>(operation: () => Promise<T>): T {
        operations.push(operation);
        return null as any; // 类型占位符
      },

      /**
       * 执行所有批次操作
       */
      async execute(): Promise<any[]> {
        const results = await Promise.allSettled(
          operations.map(op => op())
        );

        return results.map(result => 
          result.status === 'fulfilled' ? result.value : result.reason
        );
      },

      /**
       * 并行执行所有操作（失败时不影响其他操作）
       */
      async executeParallel(): Promise<{
        successful: any[];
        failed: { index: number; error: any }[];
      }> {
        const results = await Promise.allSettled(
          operations.map(op => op())
        );

        const successful: any[] = [];
        const failed: { index: number; error: any }[] = [];

        results.forEach((result, index) => {
          if (result.status === 'fulfilled') {
            successful.push(result.value);
          } else {
            failed.push({ index, error: result.reason });
          }
        });

        return { successful, failed };
      },
    };
  }

  /**
   * 工具方法：格式化错误信息
   */
  public static formatError(error: any): {
    message: string;
    code?: string;
    details?: any;
  } {
    if (error instanceof ArbitrageSDKError) {
      return {
        message: error.message,
        code: error.code,
        details: error.details,
      };
    }

    return {
      message: error?.message || '未知错误',
      details: error,
    };
  }

  /**
   * 工具方法：等待指定时间
   */
  public static async wait(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * 工具方法：重试执行
   */
  public static async retry<T>(
    operation: () => Promise<T>,
    maxAttempts: number = 3,
    delayMs: number = 1000
  ): Promise<T> {
    let lastError: Error;

    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        return await operation();
      } catch (error) {
        lastError = error as Error;
        
        if (attempt === maxAttempts) {
          break;
        }

        await this.wait(delayMs * attempt);
      }
    }

    throw lastError!;
  }
}