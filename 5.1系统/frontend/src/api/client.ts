// 基础API客户端
import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import type { ApiResponse, ApiError, ApiRequestConfig, HttpClient } from '@/types/api';

class APIClient implements HttpClient {
  private client: AxiosInstance;
  private baseURL: string;
  private timeout: number;

  constructor() {
    this.baseURL = import.meta.env.VITE_API_BASE_URL || '';
    this.timeout = parseInt(import.meta.env.VITE_API_TIMEOUT) || 10000;
    
    console.log('🔧 API客户端初始化:');
    console.log('📍 baseURL:', this.baseURL || '(使用相对路径)');
    console.log('⏱️ timeout:', this.timeout);

    this.client = axios.create({
      baseURL: this.baseURL,
      timeout: this.timeout,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    this.setupInterceptors();
  }

  private setupInterceptors(): void {
    // 请求拦截器
    this.client.interceptors.request.use(
      (config) => {
        // 添加认证头
        const token = this.getAuthToken();
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }

        // 添加请求ID
        config.headers['X-Request-ID'] = this.generateRequestId();

        // 添加时间戳
        config.headers['X-Timestamp'] = Date.now().toString();

        console.log(`🚀 API Request: ${config.method?.toUpperCase()} ${config.url}`);
        return config;
      },
      (error) => {
        console.error('❌ Request Error:', error);
        return Promise.reject(error);
      }
    );

    // 响应拦截器
    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        console.log(`✅ API Response: ${response.status} ${response.config.url}`);
        return response;
      },
      (error) => {
        console.error(`❌ API Error: ${error.response?.status} ${error.config?.url}`, error.response?.data);
        
        // 处理认证错误
        if (error.response?.status === 401) {
          this.handleAuthError();
        }

        // 处理网络错误
        if (!error.response) {
          console.error('❌ Network Error: 无法连接到服务器');
        }

        return Promise.reject(this.normalizeError(error));
      }
    );
  }

  private getAuthToken(): string | null {
    return localStorage.getItem('auth_token');
  }

  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  private handleAuthError(): void {
    // 清除本地存储的token
    localStorage.removeItem('auth_token');
    // 重定向到登录页
    window.location.href = '/login';
  }

  private normalizeError(error: any): ApiError {
    return {
      code: error.response?.data?.code || error.code || 'UNKNOWN_ERROR',
      message: error.response?.data?.message || error.message || '未知错误',
      details: error.response?.data?.details || {},
      timestamp: new Date().toISOString(),
    };
  }

  // HTTP方法实现
  async get<T = any>(url: string, config?: ApiRequestConfig): Promise<T> {
    const response = await this.client.get<ApiResponse<T>>(url, this.mergeConfig(config));
    return this.extractData(response.data);
  }

  async post<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T> {
    const response = await this.client.post<ApiResponse<T>>(url, data, this.mergeConfig(config));
    return this.extractData(response.data);
  }

  async put<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T> {
    const response = await this.client.put<ApiResponse<T>>(url, data, this.mergeConfig(config));
    return this.extractData(response.data);
  }

  async delete<T = any>(url: string, config?: ApiRequestConfig): Promise<T> {
    const response = await this.client.delete<ApiResponse<T>>(url, this.mergeConfig(config));
    return this.extractData(response.data);
  }

  async patch<T = any>(url: string, data?: any, config?: ApiRequestConfig): Promise<T> {
    const response = await this.client.patch<ApiResponse<T>>(url, data, this.mergeConfig(config));
    return this.extractData(response.data);
  }

  // 批量请求
  async batch<T = any>(requests: Array<{
    method: 'get' | 'post' | 'put' | 'delete' | 'patch';
    url: string;
    data?: any;
    config?: ApiRequestConfig;
  }>): Promise<T[]> {
    const promises = requests.map(req => {
      switch (req.method) {
        case 'get':
          return this.get(req.url, req.config);
        case 'post':
          return this.post(req.url, req.data, req.config);
        case 'put':
          return this.put(req.url, req.data, req.config);
        case 'delete':
          return this.delete(req.url, req.config);
        case 'patch':
          return this.patch(req.url, req.data, req.config);
        default:
          throw new Error(`不支持的HTTP方法: ${req.method}`);
      }
    });

    return Promise.all(promises);
  }

  // 上传文件
  async upload<T = any>(url: string, file: File, config?: ApiRequestConfig): Promise<T> {
    const formData = new FormData();
    formData.append('file', file);

    const uploadConfig = {
      ...config,
      headers: {
        'Content-Type': 'multipart/form-data',
        ...config?.headers,
      },
    };

    const response = await this.client.post<ApiResponse<T>>(url, formData, this.mergeConfig(uploadConfig));
    return this.extractData(response.data);
  }

  // 下载文件
  async download(url: string, filename?: string, config?: ApiRequestConfig): Promise<void> {
    const response = await this.client.get(url, {
      ...this.mergeConfig(config),
      responseType: 'blob',
    });

    const blob = new Blob([response.data]);
    const downloadUrl = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = downloadUrl;
    link.download = filename || 'download';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(downloadUrl);
  }

  private mergeConfig(config?: ApiRequestConfig): AxiosRequestConfig {
    return {
      timeout: config?.timeout || this.timeout,
      headers: config?.headers || {},
      params: config?.params || {},
    };
  }

  private extractData<T>(response: any): T {
    // 如果响应有success字段，检查它
    if (response.hasOwnProperty('success') && !response.success) {
      throw new Error(response.error || response.message || '请求失败');
    }
    // 如果有data字段，返回data，否则返回整个响应
    return response.data || response;
  }

  // 健康检查
  async healthCheck(): Promise<boolean> {
    try {
      console.log('🩺 执行健康检查...');
      // 直接使用axios而不通过get方法，避免数据提取问题
      const response = await this.client.get('/health');
      console.log('🩺 健康检查响应:', response.data);
      const isHealthy = response.data?.status === 'ok';
      console.log('🩺 健康状态:', isHealthy);
      return isHealthy;
    } catch (error) {
      console.error('❌ Health check failed:', error);
      return false;
    }
  }

  // 获取API版本信息
  async getVersion(): Promise<{ version: string; build: string; timestamp: string }> {
    return this.get('/version');
  }

  // 获取系统状态
  async getSystemStatus(): Promise<any> {
    try {
      console.log('📊 获取系统状态...');
      // 直接使用axios而不通过get方法
      const response = await this.client.get('/api/system/status');
      console.log('📊 系统状态响应:', response.data);
      return response.data;
    } catch (error) {
      console.error('❌ System status check failed:', error);
      return {};
    }
  }
}

// 创建单例实例
export const apiClient = new APIClient();

// 导出类型和实例
export type { APIClient };
export default apiClient;