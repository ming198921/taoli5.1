import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';

// API客户端配置
const API_BASE_URL = 'http://localhost:3000/api';
const WS_BASE_URL = 'ws://localhost:3000/ws';

// API重试配置
const RETRY_CONFIG = {
  retries: 2,
  retryDelay: 1000,
  retryCondition: (error: any) => {
    // 404, 401, 403等客户端错误不重试
    if (error.response?.status && error.response.status >= 400 && error.response.status < 500) {
      return false;
    }
    // 只对网络错误或5xx服务器错误重试
    return !error.response || (error.response.status >= 500 && error.response.status < 600) || error.code === 'ECONNABORTED';
  }
};

// 创建axios实例
export const apiClient: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器 - 添加认证和日志
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    
    // 添加请求ID用于追踪
    config.headers['X-Request-ID'] = generateRequestId();
    
    console.log(`🔄 API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('❌ Request Error:', error);
    return Promise.reject(error);
  }
);

// 响应拦截器 - 统一错误处理和重试逻辑
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    console.log(`✅ API Response: ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status}`);
    // 重置重试计数
    if ((response.config as any).__retryCount) {
      delete (response.config as any).__retryCount;
    }
    return response;
  },
  async (error) => {
    const config = error.config;
    
    console.error('❌ API Error:', {
      method: config?.method?.toUpperCase(),
      url: config?.url,
      status: error.response?.status,
      data: error.response?.data,
      retryCount: (config as any)?.__retryCount || 0
    });
    
    // 重试逻辑
    if (config && RETRY_CONFIG.retryCondition(error)) {
      (config as any).__retryCount = (config as any).__retryCount || 0;
      
      if ((config as any).__retryCount < RETRY_CONFIG.retries) {
        (config as any).__retryCount += 1;
        
        console.warn(`🔄 Retrying API request (${(config as any).__retryCount}/${RETRY_CONFIG.retries}): ${config.method?.toUpperCase()} ${config.url}`);
        
        // 等待重试延迟
        await new Promise(resolve => setTimeout(resolve, RETRY_CONFIG.retryDelay * (config as any).__retryCount));
        
        return apiClient.request(config);
      }
    }
    
    // 统一错误处理
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    } else if (error.response?.status === 503) {
      console.warn('⚠️ Service temporarily unavailable, please try again later');
    }
    
    return Promise.reject(error);
  }
);

// 生成请求ID
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

// WebSocket连接管理器
export class WebSocketManager {
  private connections: Map<string, WebSocket> = new Map();
  
  connect(endpoint: string, onMessage?: (data: any) => void, onError?: (error: any) => void): WebSocket {
    const wsUrl = `${WS_BASE_URL}${endpoint}`;
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log(`🔗 WebSocket Connected: ${endpoint}`);
    };
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onMessage?.(data);
      } catch (error) {
        console.error('WebSocket message parse error:', error);
      }
    };
    
    ws.onerror = (error) => {
      console.error(`❌ WebSocket Error: ${endpoint}`, error);
      onError?.(error);
    };
    
    ws.onclose = () => {
      console.log(`🔌 WebSocket Disconnected: ${endpoint}`);
      this.connections.delete(endpoint);
      
      // 自动重连逻辑
      setTimeout(() => {
        console.log(`🔄 Reconnecting WebSocket: ${endpoint}`);
        this.connect(endpoint, onMessage, onError);
      }, 5000);
    };
    
    this.connections.set(endpoint, ws);
    return ws;
  }
  
  disconnect(endpoint: string) {
    const ws = this.connections.get(endpoint);
    if (ws) {
      ws.close();
      this.connections.delete(endpoint);
    }
  }
  
  disconnectAll() {
    this.connections.forEach(ws => ws.close());
    this.connections.clear();
  }
}

// 全局WebSocket管理器实例
export const wsManager = new WebSocketManager();

// API响应类型
export interface ApiResponse<T = any> {
  success: boolean;
  data: T;
  message?: string;
  timestamp?: string;
}

// HTTP方法枚举
export enum HttpMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  DELETE = 'DELETE',
}

// 通用API调用函数
export async function apiCall<T = any>(
  method: HttpMethod,
  endpoint: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  try {
    const response = await apiClient.request({
      method,
      url: endpoint,
      data,
      ...config,
    });
    return response.data;
  } catch (error) {
    throw error;
  }
}

export default apiClient; 

// API客户端配置
const API_BASE_URL = 'http://localhost:3000/api';
const WS_BASE_URL = 'ws://localhost:3000/ws';

// API重试配置
const RETRY_CONFIG = {
  retries: 2,
  retryDelay: 1000,
  retryCondition: (error: any) => {
    // 404, 401, 403等客户端错误不重试
    if (error.response?.status && error.response.status >= 400 && error.response.status < 500) {
      return false;
    }
    // 只对网络错误或5xx服务器错误重试
    return !error.response || (error.response.status >= 500 && error.response.status < 600) || error.code === 'ECONNABORTED';
  }
};

// 创建axios实例
export const apiClient: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器 - 添加认证和日志
apiClient.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    
    // 添加请求ID用于追踪
    config.headers['X-Request-ID'] = generateRequestId();
    
    console.log(`🔄 API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => {
    console.error('❌ Request Error:', error);
    return Promise.reject(error);
  }
);

// 响应拦截器 - 统一错误处理和重试逻辑
apiClient.interceptors.response.use(
  (response: AxiosResponse) => {
    console.log(`✅ API Response: ${response.config.method?.toUpperCase()} ${response.config.url} - ${response.status}`);
    // 重置重试计数
    if ((response.config as any).__retryCount) {
      delete (response.config as any).__retryCount;
    }
    return response;
  },
  async (error) => {
    const config = error.config;
    
    console.error('❌ API Error:', {
      method: config?.method?.toUpperCase(),
      url: config?.url,
      status: error.response?.status,
      data: error.response?.data,
      retryCount: (config as any)?.__retryCount || 0
    });
    
    // 重试逻辑
    if (config && RETRY_CONFIG.retryCondition(error)) {
      (config as any).__retryCount = (config as any).__retryCount || 0;
      
      if ((config as any).__retryCount < RETRY_CONFIG.retries) {
        (config as any).__retryCount += 1;
        
        console.warn(`🔄 Retrying API request (${(config as any).__retryCount}/${RETRY_CONFIG.retries}): ${config.method?.toUpperCase()} ${config.url}`);
        
        // 等待重试延迟
        await new Promise(resolve => setTimeout(resolve, RETRY_CONFIG.retryDelay * (config as any).__retryCount));
        
        return apiClient.request(config);
      }
    }
    
    // 统一错误处理
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      window.location.href = '/login';
    } else if (error.response?.status === 503) {
      console.warn('⚠️ Service temporarily unavailable, please try again later');
    }
    
    return Promise.reject(error);
  }
);

// 生成请求ID
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substring(2, 11)}`;
}

// WebSocket连接管理器
export class WebSocketManager {
  private connections: Map<string, WebSocket> = new Map();
  
  connect(endpoint: string, onMessage?: (data: any) => void, onError?: (error: any) => void): WebSocket {
    const wsUrl = `${WS_BASE_URL}${endpoint}`;
    const ws = new WebSocket(wsUrl);
    
    ws.onopen = () => {
      console.log(`🔗 WebSocket Connected: ${endpoint}`);
    };
    
    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        onMessage?.(data);
      } catch (error) {
        console.error('WebSocket message parse error:', error);
      }
    };
    
    ws.onerror = (error) => {
      console.error(`❌ WebSocket Error: ${endpoint}`, error);
      onError?.(error);
    };
    
    ws.onclose = () => {
      console.log(`🔌 WebSocket Disconnected: ${endpoint}`);
      this.connections.delete(endpoint);
      
      // 自动重连逻辑
      setTimeout(() => {
        console.log(`🔄 Reconnecting WebSocket: ${endpoint}`);
        this.connect(endpoint, onMessage, onError);
      }, 5000);
    };
    
    this.connections.set(endpoint, ws);
    return ws;
  }
  
  disconnect(endpoint: string) {
    const ws = this.connections.get(endpoint);
    if (ws) {
      ws.close();
      this.connections.delete(endpoint);
    }
  }
  
  disconnectAll() {
    this.connections.forEach(ws => ws.close());
    this.connections.clear();
  }
}

// 全局WebSocket管理器实例
export const wsManager = new WebSocketManager();

// API响应类型
export interface ApiResponse<T = any> {
  success: boolean;
  data: T;
  message?: string;
  timestamp?: string;
}

// HTTP方法枚举
export enum HttpMethod {
  GET = 'GET',
  POST = 'POST',
  PUT = 'PUT',
  DELETE = 'DELETE',
}

// 通用API调用函数
export async function apiCall<T = any>(
  method: HttpMethod,
  endpoint: string,
  data?: any,
  config?: AxiosRequestConfig
): Promise<T> {
  try {
    const response = await apiClient.request({
      method,
      url: endpoint,
      data,
      ...config,
    });
    return response.data;
  } catch (error) {
    throw error;
  }
}

export default apiClient; 