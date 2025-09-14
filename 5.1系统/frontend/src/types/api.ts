// API相关类型定义
export interface ApiResponse<T = any> {
  success: boolean;
  data?: T;
  error?: string;
  message?: string;
  timestamp?: number;
}

export interface ApiRequestConfig {
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
  params?: Record<string, any>;
}

export interface WebSocketConfig {
  url: string;
  autoReconnect: boolean;
  reconnectInterval: number;
  maxReconnectAttempts: number;
  pingInterval: number;
}

export interface WebSocketMessage<T = any> {
  type: string;
  data: T;
  timestamp: string;
  id?: string;
}

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, any>;
  timestamp: string;
}

export interface SubscriptionConfig {
  topic: string;
  params?: Record<string, any>;
  qos?: 0 | 1 | 2;
}

export interface StreamingData<T = any> {
  topic: string;
  data: T;
  sequence: number;
  timestamp: string;
}

// HTTP客户端接口
export interface HttpClient {
  get<T = any>(url: string, config?: ApiRequestConfig): Promise<T>;
  post<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T>;
  put<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T>;
  delete<T = any>(url: string, config?: ApiRequestConfig): Promise<T>;
  patch<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T>;
}

// WebSocket客户端接口
export interface WebSocketClient {
  connect(): Promise<void>;
  disconnect(): void;
  subscribe<T = any>(topic: string, callback: (data: T) => void): void;
  unsubscribe(topic: string): void;
  send(message: any): void;
  isConnected(): boolean;
}

// 实时数据流类型
export interface DataStream<T = any> {
  subscribe(callback: (data: T) => void): () => void;
  unsubscribe(): void;
  getLatest(): T | null;
  isActive(): boolean;
}