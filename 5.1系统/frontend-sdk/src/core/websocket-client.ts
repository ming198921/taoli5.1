/**
 * WebSocket客户端 - 负责实时数据连接和事件处理
 */

import WebSocket from 'ws';
import { 
  WebSocketMessage, 
  EventCallback, 
  EventSubscription,
  ArbitrageSDKError,
  SDKConfig 
} from '../types';

export class WebSocketClient {
  private ws?: WebSocket;
  private config: SDKConfig;
  private wsUrl: string;
  private isConnected: boolean = false;
  private isConnecting: boolean = false;
  private reconnectAttempts: number = 0;
  private maxReconnectAttempts: number = 5;
  private reconnectDelay: number = 1000;
  private heartbeatInterval?: NodeJS.Timeout;
  private authToken?: string;
  
  // 事件监听器管理
  private eventListeners: Map<string, Set<EventCallback>> = new Map();
  private subscriptions: Map<string, EventSubscription> = new Map();

  constructor(config: SDKConfig) {
    this.config = config;
    this.wsUrl = config.wsUrl || config.baseUrl.replace(/^http/, 'ws') + '/ws';
    
    if (this.config.enableLogging) {
      console.log('[WebSocket] Initialized with URL:', this.wsUrl);
    }
  }

  /**
   * 连接WebSocket服务器
   */
  public async connect(authToken?: string): Promise<void> {
    if (this.isConnected || this.isConnecting) {
      return;
    }

    this.isConnecting = true;
    this.authToken = authToken;

    return new Promise((resolve, reject) => {
      try {
        const wsUrl = this.buildWsUrl();
        this.ws = new WebSocket(wsUrl);

        // 连接成功
        this.ws.onopen = () => {
          this.isConnected = true;
          this.isConnecting = false;
          this.reconnectAttempts = 0;

          if (this.config.enableLogging) {
            console.log('[WebSocket] Connected successfully');
          }

          this.startHeartbeat();
          this.emit('connected', {});
          resolve();
        };

        // 接收消息
        this.ws.onmessage = (event) => {
          this.handleMessage(event.data);
        };

        // 连接关闭
        this.ws.onclose = (event) => {
          this.isConnected = false;
          this.isConnecting = false;
          this.stopHeartbeat();

          if (this.config.enableLogging) {
            console.log('[WebSocket] Connection closed:', event.code, event.reason);
          }

          this.emit('disconnected', { code: event.code, reason: event.reason });

          // 自动重连
          if (event.code !== 1000 && this.reconnectAttempts < this.maxReconnectAttempts) {
            this.scheduleReconnect();
          }
        };

        // 连接错误
        this.ws.onerror = (error) => {
          if (this.config.enableLogging) {
            console.error('[WebSocket] Connection error:', error);
          }

          this.emit('error', error);

          if (this.isConnecting) {
            this.isConnecting = false;
            reject(new ArbitrageSDKError(
              'WebSocket连接失败',
              'WS_CONNECTION_ERROR',
              error
            ));
          }
        };
      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  /**
   * 断开WebSocket连接
   */
  public disconnect(): void {
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
    }
    this.stopHeartbeat();
    this.isConnected = false;
    this.isConnecting = false;
  }

  /**
   * 发送消息
   */
  public send(message: any): void {
    if (!this.isConnected || !this.ws) {
      throw new ArbitrageSDKError(
        'WebSocket未连接',
        'WS_NOT_CONNECTED'
      );
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

  /**
   * 订阅事件
   */
  public subscribe<T = any>(
    eventType: string, 
    callback: EventCallback<T>
  ): EventSubscription {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }

    this.eventListeners.get(eventType)!.add(callback);

    const subscription: EventSubscription = {
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

    // 如果连接已建立，发送订阅消息到服务器
    if (this.isConnected) {
      this.send({
        type: 'subscribe',
        event: eventType,
      });
    }

    return subscription;
  }

  /**
   * 取消订阅事件
   */
  public unsubscribe(eventType: string): void {
    this.eventListeners.delete(eventType);

    if (this.isConnected) {
      this.send({
        type: 'unsubscribe',
        event: eventType,
      });
    }
  }

  /**
   * 订阅市场数据
   */
  public subscribeMarketData(callback: EventCallback): EventSubscription {
    return this.subscribe('market_data', callback);
  }

  /**
   * 订阅订单簿数据
   */
  public subscribeOrderBook(callback: EventCallback): EventSubscription {
    return this.subscribe('orderbook', callback);
  }

  /**
   * 订阅套利机会
   */
  public subscribeArbitrageOpportunities(callback: EventCallback): EventSubscription {
    return this.subscribe('arbitrage_opportunity', callback);
  }

  /**
   * 订阅系统警报
   */
  public subscribeAlerts(callback: EventCallback): EventSubscription {
    return this.subscribe('alert', callback);
  }

  /**
   * 订阅系统状态更新
   */
  public subscribeSystemStatus(callback: EventCallback): EventSubscription {
    return this.subscribe('system_status', callback);
  }

  /**
   * 获取连接状态
   */
  public getConnectionStatus(): {
    connected: boolean;
    connecting: boolean;
    reconnectAttempts: number;
  } {
    return {
      connected: this.isConnected,
      connecting: this.isConnecting,
      reconnectAttempts: this.reconnectAttempts,
    };
  }

  /**
   * 设置认证令牌
   */
  public setAuthToken(token: string): void {
    this.authToken = token;
    
    // 如果已连接，发送认证消息
    if (this.isConnected) {
      this.send({
        type: 'auth',
        token: token,
      });
    }
  }

  /**
   * 处理接收到的消息
   */
  private handleMessage(data: string): void {
    try {
      const message: WebSocketMessage = JSON.parse(data);

      if (this.config.enableLogging) {
        console.log('[WebSocket] Received message:', message);
      }

      // 处理特殊消息类型
      switch (message.type) {
        case 'pong':
          // 心跳响应，不需要处理
          return;
        case 'auth_success':
          this.emit('authenticated', message.data);
          return;
        case 'auth_failed':
          this.emit('auth_failed', message.data);
          return;
        case 'error':
          this.emit('error', new ArbitrageSDKError(
            message.data.message || '服务器错误',
            message.data.code || 'SERVER_ERROR',
            message.data
          ));
          return;
      }

      // 分发到相应的事件监听器
      this.emit(message.type, message.data);
    } catch (error) {
      if (this.config.enableLogging) {
        console.error('[WebSocket] Failed to parse message:', data, error);
      }
      
      this.emit('error', new ArbitrageSDKError(
        '消息解析失败',
        'MESSAGE_PARSE_ERROR',
        { data, error }
      ));
    }
  }

  /**
   * 触发事件
   */
  private emit(eventType: string, data: any): void {
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

  /**
   * 构建WebSocket URL
   */
  private buildWsUrl(): string {
    const url = new URL(this.wsUrl);
    
    if (this.authToken) {
      url.searchParams.set('token', this.authToken);
    }
    
    return url.toString();
  }

  /**
   * 开始心跳
   */
  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      if (this.isConnected) {
        this.send({ type: 'ping' });
      }
    }, 30000); // 每30秒发送一次心跳
  }

  /**
   * 停止心跳
   */
  private stopHeartbeat(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = undefined;
    }
  }

  /**
   * 安排重连
   */
  private scheduleReconnect(): void {
    this.reconnectAttempts++;
    const delay = Math.min(this.reconnectDelay * Math.pow(2, this.reconnectAttempts), 30000);

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
}