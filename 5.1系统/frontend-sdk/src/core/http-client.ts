/**
 * HTTP客户端 - 负责所有API请求的统一处理
 */

import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { ApiResponse, ApiError, ArbitrageSDKError, SDKConfig } from '../types';

export class HttpClient {
  private client: AxiosInstance;
  private config: SDKConfig;
  private authToken?: string;

  constructor(config: SDKConfig) {
    this.config = config;
    this.client = axios.create({
      baseURL: config.baseUrl,
      timeout: config.timeout || 30000,
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json',
      },
    });

    this.setupInterceptors();
  }

  /**
   * 设置请求和响应拦截器
   */
  private setupInterceptors(): void {
    // 请求拦截器 - 添加认证令牌
    this.client.interceptors.request.use(
      (config) => {
        // 添加认证令牌
        if (this.authToken) {
          config.headers.Authorization = `Bearer ${this.authToken}`;
        }

        // 添加API密钥
        if (this.config.apiKey) {
          config.headers['X-API-Key'] = this.config.apiKey;
        }

        // 添加请求ID用于追踪
        config.headers['X-Request-ID'] = this.generateRequestId();

        // 日志记录
        if (this.config.enableLogging) {
          console.log(`[HTTP] ${config.method?.toUpperCase()} ${config.url}`, {
            data: config.data,
            headers: this.sanitizeHeaders(config.headers),
          });
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

    // 响应拦截器 - 统一错误处理
    this.client.interceptors.response.use(
      (response: AxiosResponse) => {
        // 日志记录
        if (this.config.enableLogging) {
          console.log(`[HTTP] Response ${response.status}:`, {
            url: response.config.url,
            data: response.data,
          });
        }

        return response;
      },
      async (error) => {
        const originalRequest = error.config;

        // 日志记录
        if (this.config.enableLogging) {
          console.error('[HTTP] Response Error:', {
            status: error.response?.status,
            url: originalRequest?.url,
            data: error.response?.data,
          });
        }

        // 401错误 - 令牌过期，尝试刷新
        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true;

          try {
            await this.refreshAuthToken();
            return this.client(originalRequest);
          } catch (refreshError) {
            // 刷新失败，清除令牌
            this.clearAuthToken();
            throw new ArbitrageSDKError(
              '认证失败，请重新登录',
              'AUTH_FAILED',
              refreshError
            );
          }
        }

        // 网络错误
        if (!error.response) {
          throw new ArbitrageSDKError(
            '网络连接失败',
            'NETWORK_ERROR',
            error
          );
        }

        // API错误
        const apiError: ApiError = error.response.data;
        throw new ArbitrageSDKError(
          apiError.message || '服务器错误',
          apiError.code || 'SERVER_ERROR',
          apiError.details
        );
      }
    );
  }

  /**
   * 设置认证令牌
   */
  public setAuthToken(token: string): void {
    this.authToken = token;
  }

  /**
   * 清除认证令牌
   */
  public clearAuthToken(): void {
    this.authToken = undefined;
  }

  /**
   * 刷新认证令牌
   */
  private async refreshAuthToken(): Promise<void> {
    const refreshToken = this.getStoredRefreshToken();
    if (!refreshToken) {
      throw new Error('没有刷新令牌');
    }

    const response = await this.post<{ token: string }>('/api/auth/refresh', {
      refresh_token: refreshToken,
    });

    if (response.success && response.data) {
      this.setAuthToken(response.data.token);
      this.storeAuthToken(response.data.token);
    } else {
      throw new Error('刷新令牌失败');
    }
  }

  /**
   * GET请求
   */
  public async get<T>(
    url: string,
    config?: AxiosRequestConfig
  ): Promise<ApiResponse<T>> {
    const response = await this.client.get(url, config);
    return response.data;
  }

  /**
   * POST请求
   */
  public async post<T>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<ApiResponse<T>> {
    const response = await this.client.post(url, data, config);
    return response.data;
  }

  /**
   * PUT请求
   */
  public async put<T>(
    url: string,
    data?: any,
    config?: AxiosRequestConfig
  ): Promise<ApiResponse<T>> {
    const response = await this.client.put(url, data, config);
    return response.data;
  }

  /**
   * DELETE请求
   */
  public async delete<T>(
    url: string,
    config?: AxiosRequestConfig
  ): Promise<ApiResponse<T>> {
    const response = await this.client.delete(url, config);
    return response.data;
  }

  /**
   * 带重试的请求
   */
  public async requestWithRetry<T>(
    requestFn: () => Promise<ApiResponse<T>>,
    maxRetries: number = this.config.retryAttempts || 3,
    delayMs: number = this.config.retryDelay || 1000
  ): Promise<ApiResponse<T>> {
    let lastError: Error;

    for (let attempt = 1; attempt <= maxRetries; attempt++) {
      try {
        return await requestFn();
      } catch (error) {
        lastError = error as Error;

        if (attempt === maxRetries) {
          break;
        }

        // 某些错误不应该重试
        if (error instanceof ArbitrageSDKError) {
          if (['AUTH_FAILED', 'VALIDATION_ERROR'].includes(error.code)) {
            break;
          }
        }

        // 等待后重试
        await this.delay(delayMs * attempt);

        if (this.config.enableLogging) {
          console.warn(`[HTTP] Retry attempt ${attempt}/${maxRetries}`);
        }
      }
    }

    throw lastError!;
  }

  /**
   * 生成请求ID
   */
  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 清理敏感头部信息用于日志记录
   */
  private sanitizeHeaders(headers: any): any {
    const sanitized = { ...headers };
    if (sanitized.Authorization) {
      sanitized.Authorization = 'Bearer ***';
    }
    if (sanitized['X-API-Key']) {
      sanitized['X-API-Key'] = '***';
    }
    return sanitized;
  }

  /**
   * 延迟函数
   */
  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }

  /**
   * 存储认证令牌
   */
  private storeAuthToken(token: string): void {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('arbitrage_auth_token', token);
    }
  }

  /**
   * 获取存储的刷新令牌
   */
  private getStoredRefreshToken(): string | null {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('arbitrage_refresh_token');
    }
    return null;
  }

  /**
   * 存储刷新令牌
   */
  public storeRefreshToken(refreshToken: string): void {
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('arbitrage_refresh_token', refreshToken);
    }
  }

  /**
   * 清除存储的令牌
   */
  public clearStoredTokens(): void {
    if (typeof localStorage !== 'undefined') {
      localStorage.removeItem('arbitrage_auth_token');
      localStorage.removeItem('arbitrage_refresh_token');
    }
  }

  /**
   * 获取存储的认证令牌
   */
  public getStoredAuthToken(): string | null {
    if (typeof localStorage !== 'undefined') {
      return localStorage.getItem('arbitrage_auth_token');
    }
    return null;
  }
}