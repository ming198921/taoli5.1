/**
 * 系统服务 - 系统控制和状态管理
 */

import { HttpClient } from '../core/http-client';
import { SystemStatus, ApiResponse } from '../types';

export class SystemService {
  constructor(private httpClient: HttpClient) {}

  /**
   * 获取系统状态
   */
  public async getSystemStatus(): Promise<SystemStatus> {
    const response = await this.httpClient.get<SystemStatus>('/api/system/status');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统状态失败');
  }

  /**
   * 启动系统
   */
  public async startSystem(): Promise<{
    status: string;
    message: string;
  }> {
    const response = await this.httpClient.post<{
      status: string;
      message: string;
    }>('/api/system/start');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '启动系统失败');
  }

  /**
   * 停止系统
   */
  public async stopSystem(): Promise<{
    status: string;
    message: string;
  }> {
    const response = await this.httpClient.post<{
      status: string;
      message: string;
    }>('/api/system/stop');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '停止系统失败');
  }

  /**
   * 重启系统
   */
  public async restartSystem(): Promise<{
    status: string;
    message: string;
  }> {
    const response = await this.httpClient.post<{
      status: string;
      message: string;
    }>('/api/system/restart');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '重启系统失败');
  }

  /**
   * 获取系统版本信息
   */
  public async getSystemVersion(): Promise<{
    version: string;
    buildTime: string;
    gitCommit: string;
    dependencies: Record<string, string>;
  }> {
    const response = await this.httpClient.get('/api/system/version');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统版本失败');
  }

  /**
   * 获取系统配置
   */
  public async getSystemConfig(): Promise<Record<string, any>> {
    const response = await this.httpClient.get('/api/system/config');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统配置失败');
  }

  /**
   * 更新系统配置
   */
  public async updateSystemConfig(config: Record<string, any>): Promise<void> {
    const response = await this.httpClient.put('/api/system/config', config);
    
    if (!response.success) {
      throw new Error(response.message || '更新系统配置失败');
    }
  }

  /**
   * 健康检查
   */
  public async healthCheck(): Promise<{
    status: string;
    timestamp: number;
    uptime: number;
    version: string;
  }> {
    const response = await this.httpClient.get('/health');
    
    // 健康检查端点可能直接返回数据，不包装在ApiResponse中
    if (response) {
      return response as any;
    }
    
    throw new Error('健康检查失败');
  }

  /**
   * 获取系统日志
   */
  public async getSystemLogs(params?: {
    level?: 'debug' | 'info' | 'warn' | 'error';
    component?: string;
    limit?: number;
    offset?: number;
    startTime?: string;
    endTime?: string;
  }): Promise<{
    logs: {
      timestamp: string;
      level: string;
      component: string;
      message: string;
      metadata?: Record<string, any>;
    }[];
    total: number;
    hasMore: boolean;
  }> {
    const response = await this.httpClient.get('/api/system/logs', { params });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统日志失败');
  }

  /**
   * 清理系统日志
   */
  public async clearLogs(params?: {
    olderThan?: string;
    level?: string;
    component?: string;
  }): Promise<{
    deletedCount: number;
    message: string;
  }> {
    const response = await this.httpClient.delete('/api/system/logs', {
      data: params,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '清理系统日志失败');
  }

  /**
   * 获取系统统计
   */
  public async getSystemStats(): Promise<{
    uptime: number;
    startTime: string;
    processId: number;
    memoryUsage: {
      rss: number;
      heapTotal: number;
      heapUsed: number;
      external: number;
    };
    cpuUsage: {
      user: number;
      system: number;
    };
    activeConnections: number;
    totalRequests: number;
    averageResponseTime: number;
  }> {
    const response = await this.httpClient.get('/api/system/stats');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统统计失败');
  }

  /**
   * 执行系统维护任务
   */
  public async performMaintenance(tasks: {
    clearCache?: boolean;
    cleanupLogs?: boolean;
    optimizeDatabase?: boolean;
    updateDependencies?: boolean;
  }): Promise<{
    taskId: string;
    status: 'running' | 'completed' | 'failed';
    progress: number;
    results: Record<string, any>;
  }> {
    const response = await this.httpClient.post('/api/system/maintenance', tasks);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '执行系统维护失败');
  }

  /**
   * 获取维护任务状态
   */
  public async getMaintenanceStatus(taskId: string): Promise<{
    taskId: string;
    status: 'running' | 'completed' | 'failed';
    progress: number;
    startTime: string;
    endTime?: string;
    results: Record<string, any>;
    error?: string;
  }> {
    const response = await this.httpClient.get(`/api/system/maintenance/${taskId}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取维护任务状态失败');
  }

  /**
   * 备份系统数据
   */
  public async backupSystem(params: {
    includeConfig?: boolean;
    includeLogs?: boolean;
    includeUserData?: boolean;
    compressionLevel?: 1 | 2 | 3 | 4 | 5;
  }): Promise<{
    backupId: string;
    status: 'running' | 'completed' | 'failed';
    downloadUrl?: string;
  }> {
    const response = await this.httpClient.post('/api/system/backup', params);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '备份系统数据失败');
  }

  /**
   * 获取备份状态
   */
  public async getBackupStatus(backupId: string): Promise<{
    backupId: string;
    status: 'running' | 'completed' | 'failed';
    progress: number;
    size?: number;
    downloadUrl?: string;
    expiresAt?: string;
  }> {
    const response = await this.httpClient.get(`/api/system/backup/${backupId}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取备份状态失败');
  }

  /**
   * 恢复系统数据
   */
  public async restoreSystem(backupId: string, options?: {
    restoreConfig?: boolean;
    restoreLogs?: boolean;
    restoreUserData?: boolean;
  }): Promise<{
    restoreId: string;
    status: 'running' | 'completed' | 'failed';
    message: string;
  }> {
    const response = await this.httpClient.post(`/api/system/restore/${backupId}`, options);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '恢复系统数据失败');
  }

  /**
   * 获取恢复状态
   */
  public async getRestoreStatus(restoreId: string): Promise<{
    restoreId: string;
    status: 'running' | 'completed' | 'failed';
    progress: number;
    message: string;
    error?: string;
  }> {
    const response = await this.httpClient.get(`/api/system/restore/${restoreId}/status`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取恢复状态失败');
  }
}