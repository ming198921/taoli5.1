// WebSocket管理器
import { io, Socket } from 'socket.io-client';
import type { 
  WebSocketClient, 
  WebSocketConfig, 
  WebSocketMessage,
  MarketData, 
  SystemEvent, 
  RiskAlert,
  SubscriptionConfig
} from '@/types';

export class WebSocketManager implements WebSocketClient {
  private socket: Socket | null = null;
  private config: WebSocketConfig;
  private subscriptions: Map<string, (data: any) => void> = new Map();
  private reconnectAttempts = 0;
  private reconnectTimer?: NodeJS.Timeout;
  private pingTimer?: NodeJS.Timeout;
  private isConnecting = false;

  constructor(config?: Partial<WebSocketConfig>) {
    this.config = {
      url: import.meta.env.VITE_WS_URL || 'ws://localhost:8080',
      autoReconnect: true,
      reconnectInterval: parseInt(import.meta.env.VITE_WS_RECONNECT_INTERVAL) || 3000,
      maxReconnectAttempts: 5,
      pingInterval: 30000,
      ...config,
    };
  }

  async connect(): Promise<void> {
    if (this.isConnecting || this.isConnected()) {
      return;
    }

    this.isConnecting = true;

    return new Promise((resolve, reject) => {
      try {
        this.socket = io(this.config.url, {
          transports: ['websocket'],
          timeout: 5000,
          autoConnect: true,
          reconnection: false, // 我们自己处理重连
        });

        this.socket.on('connect', () => {
          console.log('🔗 WebSocket连接成功');
          this.isConnecting = false;
          this.reconnectAttempts = 0;
          this.startPing();
          this.resubscribeAll();
          resolve();
        });

        this.socket.on('disconnect', (reason) => {
          console.warn('🔌 WebSocket断开连接:', reason);
          this.stopPing();
          this.handleDisconnect(reason);
        });

        this.socket.on('error', (error) => {
          console.error('❌ WebSocket错误:', error);
          this.isConnecting = false;
          reject(error);
        });

        this.socket.on('reconnect', () => {
          console.log('🔄 WebSocket重新连接成功');
          this.reconnectAttempts = 0;
        });

        // 处理服务端消息
        this.setupMessageHandlers();

      } catch (error) {
        this.isConnecting = false;
        reject(error);
      }
    });
  }

  disconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = undefined;
    }

    this.stopPing();

    if (this.socket) {
      this.socket.disconnect();
      this.socket = null;
    }

    this.subscriptions.clear();
    console.log('🔌 WebSocket已断开连接');
  }

  subscribe<T = any>(topic: string, callback: (data: T) => void): void {
    this.subscriptions.set(topic, callback);

    if (this.isConnected()) {
      this.socket?.emit('subscribe', { topic });
      this.socket?.on(topic, callback);
      console.log(`📡 已订阅主题: ${topic}`);
    }
  }

  unsubscribe(topic: string): void {
    if (this.subscriptions.has(topic)) {
      this.subscriptions.delete(topic);
      
      if (this.isConnected()) {
        this.socket?.emit('unsubscribe', { topic });
        this.socket?.off(topic);
        console.log(`📡 已取消订阅主题: ${topic}`);
      }
    }
  }

  send(message: any): void {
    if (this.isConnected()) {
      this.socket?.emit('message', message);
    } else {
      console.warn('⚠️ WebSocket未连接，无法发送消息');
    }
  }

  isConnected(): boolean {
    return this.socket?.connected ?? false;
  }

  // 便捷方法：订阅实时市场数据
  subscribeRealtimeMarketData(callback: (data: MarketData) => void): void {
    this.subscribe('market:realtime', callback);
    // 同时监听 market_update 事件
    if (this.isConnected()) {
      this.socket?.on('market_update', callback);
    }
  }

  // 便捷方法：订阅系统性能数据
  subscribePerformanceData(callback: (data: any) => void): void {
    this.subscribe('performance:realtime', callback);
    // 同时监听 performance_update 事件
    if (this.isConnected()) {
      this.socket?.on('performance_update', callback);
    }
  }

  // 便捷方法：订阅市场数据（兼容旧版本）
  subscribeMarketData(symbols: string[], callback: (data: MarketData) => void): void {
    symbols.forEach(symbol => {
      this.subscribe(`market:${symbol}`, callback);
    });
  }

  // 便捷方法：订阅系统事件
  subscribeSystemEvents(callback: (event: SystemEvent) => void): void {
    this.subscribe('system:events', callback);
  }

  // 便捷方法：订阅风险警报
  subscribeRiskAlerts(callback: (alert: RiskAlert) => void): void {
    this.subscribe('risk:alerts', callback);
  }

  // 便捷方法：订阅交易执行事件
  subscribeTradeExecution(callback: (data: any) => void): void {
    this.subscribe('trade:execution', callback);
  }

  // 便捷方法：订阅策略状态变化
  subscribeStrategyStatus(callback: (data: any) => void): void {
    this.subscribe('strategy:status', callback);
  }

  private setupMessageHandlers(): void {
    if (!this.socket) return;

    // 处理服务端推送的消息
    this.socket.on('message', (data: WebSocketMessage) => {
      console.log('📨 收到WebSocket消息:', data.type);
      
      // 根据消息类型分发到对应的处理器
      const handler = this.subscriptions.get(data.type);
      if (handler) {
        handler(data.data);
      }
    });

    // 处理ping-pong心跳
    this.socket.on('ping', () => {
      this.socket?.emit('pong');
    });

    this.socket.on('pong', () => {
      // 心跳响应，连接正常
    });
  }

  private handleDisconnect(_reason: string): void {
    if (!this.config.autoReconnect) {
      return;
    }

    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      console.error('❌ WebSocket重连次数已达上限，停止重连');
      return;
    }

    const delay = this.config.reconnectInterval * Math.pow(2, this.reconnectAttempts);
    this.reconnectAttempts++;

    console.log(`🔄 ${delay}ms后尝试重连 (第${this.reconnectAttempts}次)`);

    this.reconnectTimer = setTimeout(() => {
      this.connect().catch(error => {
        console.error('❌ WebSocket重连失败:', error);
      });
    }, delay);
  }

  private startPing(): void {
    this.pingTimer = setInterval(() => {
      if (this.isConnected()) {
        this.socket?.emit('ping');
      }
    }, this.config.pingInterval);
  }

  private stopPing(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = undefined;
    }
  }

  private resubscribeAll(): void {
    // 重新订阅所有主题
    this.subscriptions.forEach((callback, topic) => {
      this.socket?.emit('subscribe', { topic });
      this.socket?.on(topic, callback);
    });
  }

  // 获取连接状态
  getConnectionStatus(): {
    connected: boolean;
    reconnectAttempts: number;
    maxReconnectAttempts: number;
    subscriptionCount: number;
  } {
    return {
      connected: this.isConnected(),
      reconnectAttempts: this.reconnectAttempts,
      maxReconnectAttempts: this.config.maxReconnectAttempts,
      subscriptionCount: this.subscriptions.size,
    };
  }

  // 批量订阅
  batchSubscribe(subscriptions: SubscriptionConfig[]): void {
    subscriptions.forEach(({ topic, params }) => {
      if (this.isConnected()) {
        this.socket?.emit('subscribe', { topic, params });
      }
    });
  }

  // 批量取消订阅
  batchUnsubscribe(topics: string[]): void {
    topics.forEach(topic => {
      this.unsubscribe(topic);
    });
  }
}

// 创建全局WebSocket管理器实例
export const wsManager = new WebSocketManager();

// 导出类型
export type { WebSocketManager as WebSocketManagerType };
export default wsManager;