/**
 * 监控服务 - 完整的系统监控和警报管理
 */

import { HttpClient } from '../core/http-client';
import {
  SystemMetrics,
  HealthCheck,
  Alert,
  AlertRule,
  CreateAlertRuleRequest,
  ApiResponse,
  PaginationQuery,
  PaginatedResponse,
} from '../types';

export class MonitoringService {
  constructor(private httpClient: HttpClient) {}

  /**
   * 获取系统健康检查
   */
  public async getHealthChecks(): Promise<HealthCheck[]> {
    const response = await this.httpClient.get<HealthCheck[]>('/api/monitoring/health');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取健康检查失败');
  }

  /**
   * 获取特定组件健康状态
   */
  public async getComponentHealth(component: string): Promise<HealthCheck> {
    const response = await this.httpClient.get<HealthCheck>(
      `/api/monitoring/health/${component}`
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取组件健康状态失败');
  }

  /**
   * 执行健康检查
   */
  public async performHealthCheck(component?: string): Promise<HealthCheck[]> {
    const url = component 
      ? `/api/monitoring/health/${component}/check`
      : '/api/monitoring/health/check';
    
    const response = await this.httpClient.post<HealthCheck[]>(url);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '执行健康检查失败');
  }

  /**
   * 获取系统指标
   */
  public async getSystemMetrics(
    timeRange?: { startTime: string; endTime: string },
    granularity?: 'minute' | 'hour' | 'day'
  ): Promise<SystemMetrics[]> {
    const response = await this.httpClient.get<SystemMetrics[]>('/api/monitoring/metrics', {
      params: { ...timeRange, granularity },
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取系统指标失败');
  }

  /**
   * 获取当前系统指标
   */
  public async getCurrentMetrics(): Promise<SystemMetrics> {
    const response = await this.httpClient.get<SystemMetrics>('/api/monitoring/metrics/current');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取当前系统指标失败');
  }

  /**
   * 获取指标统计
   */
  public async getMetricsStats(
    metric: string,
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    metric: string;
    period: string;
    average: number;
    minimum: number;
    maximum: number;
    median: number;
    p95: number;
    p99: number;
    trend: 'increasing' | 'decreasing' | 'stable';
    dataPoints: number;
  }> {
    const response = await this.httpClient.get(`/api/monitoring/metrics/${metric}/stats`, {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取指标统计失败');
  }

  /**
   * 获取警报列表
   */
  public async getAlerts(
    query?: PaginationQuery & {
      status?: 'active' | 'acknowledged' | 'resolved';
      type?: 'info' | 'warning' | 'error' | 'critical';
      component?: string;
      timeRange?: { startTime: string; endTime: string };
    }
  ): Promise<PaginatedResponse<Alert>> {
    const response = await this.httpClient.get<PaginatedResponse<Alert>>('/api/monitoring/alerts', {
      params: query,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取警报列表失败');
  }

  /**
   * 获取警报详情
   */
  public async getAlert(alertId: string): Promise<Alert> {
    const response = await this.httpClient.get<Alert>(`/api/monitoring/alerts/${alertId}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取警报详情失败');
  }

  /**
   * 确认警报
   */
  public async acknowledgeAlert(alertId: string, note?: string): Promise<void> {
    const response = await this.httpClient.post(
      `/api/monitoring/alerts/${alertId}/acknowledge`,
      { note }
    );
    
    if (!response.success) {
      throw new Error(response.message || '确认警报失败');
    }
  }

  /**
   * 解决警报
   */
  public async resolveAlert(alertId: string, note?: string): Promise<void> {
    const response = await this.httpClient.post(
      `/api/monitoring/alerts/${alertId}/resolve`,
      { note }
    );
    
    if (!response.success) {
      throw new Error(response.message || '解决警报失败');
    }
  }

  /**
   * 批量操作警报
   */
  public async bulkAlertAction(
    alertIds: string[],
    action: 'acknowledge' | 'resolve',
    note?: string
  ): Promise<{
    successful: string[];
    failed: { alertId: string; error: string }[];
  }> {
    const response = await this.httpClient.post('/api/monitoring/alerts/bulk', {
      alertIds,
      action,
      note,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '批量操作警报失败');
  }

  /**
   * 获取警报规则列表
   */
  public async getAlertRules(
    query?: PaginationQuery & { enabled?: boolean }
  ): Promise<PaginatedResponse<AlertRule>> {
    const response = await this.httpClient.get<PaginatedResponse<AlertRule>>(
      '/api/monitoring/alert-rules',
      { params: query }
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取警报规则失败');
  }

  /**
   * 获取警报规则详情
   */
  public async getAlertRule(ruleId: string): Promise<AlertRule> {
    const response = await this.httpClient.get<AlertRule>(`/api/monitoring/alert-rules/${ruleId}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取警报规则详情失败');
  }

  /**
   * 创建警报规则
   */
  public async createAlertRule(rule: CreateAlertRuleRequest): Promise<AlertRule> {
    const response = await this.httpClient.post<AlertRule>('/api/monitoring/alert-rules', rule);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '创建警报规则失败');
  }

  /**
   * 更新警报规则
   */
  public async updateAlertRule(
    ruleId: string,
    updates: Partial<CreateAlertRuleRequest>
  ): Promise<AlertRule> {
    const response = await this.httpClient.put<AlertRule>(
      `/api/monitoring/alert-rules/${ruleId}`,
      updates
    );
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '更新警报规则失败');
  }

  /**
   * 删除警报规则
   */
  public async deleteAlertRule(ruleId: string): Promise<void> {
    const response = await this.httpClient.delete(`/api/monitoring/alert-rules/${ruleId}`);
    
    if (!response.success) {
      throw new Error(response.message || '删除警报规则失败');
    }
  }

  /**
   * 启用/禁用警报规则
   */
  public async toggleAlertRule(ruleId: string, enabled: boolean): Promise<void> {
    const response = await this.httpClient.put(`/api/monitoring/alert-rules/${ruleId}/toggle`, {
      enabled,
    });
    
    if (!response.success) {
      throw new Error(response.message || '切换警报规则状态失败');
    }
  }

  /**
   * 测试警报规则
   */
  public async testAlertRule(
    ruleId: string,
    testData?: Record<string, number>
  ): Promise<{
    triggered: boolean;
    currentValue: number;
    threshold: number;
    message: string;
  }> {
    const response = await this.httpClient.post(`/api/monitoring/alert-rules/${ruleId}/test`, {
      testData,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '测试警报规则失败');
  }

  /**
   * 获取可用指标列表
   */
  public async getAvailableMetrics(): Promise<{
    metric: string;
    description: string;
    unit: string;
    type: 'gauge' | 'counter' | 'histogram';
    tags: string[];
  }[]> {
    const response = await this.httpClient.get('/api/monitoring/metrics/available');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取可用指标失败');
  }

  /**
   * 获取监控仪表板数据
   */
  public async getMonitoringDashboard(): Promise<{
    overview: {
      totalAlerts: number;
      criticalAlerts: number;
      systemHealth: 'healthy' | 'warning' | 'critical';
      uptime: number;
      lastUpdate: string;
    };
    components: {
      component: string;
      status: 'healthy' | 'unhealthy' | 'degraded';
      uptime: number;
      responseTime: number;
      errorRate: number;
    }[];
    recentAlerts: Alert[];
    systemTrends: {
      metric: string;
      trend: 'up' | 'down' | 'stable';
      change: number;
      period: string;
    }[];
  }> {
    const response = await this.httpClient.get('/api/monitoring/dashboard');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取监控仪表板失败');
  }

  /**
   * 获取性能报告
   */
  public async getPerformanceReport(
    timeRange: { startTime: string; endTime: string }
  ): Promise<{
    period: string;
    summary: {
      averageResponseTime: number;
      maxResponseTime: number;
      errorRate: number;
      throughput: number;
      availability: number;
    };
    bottlenecks: {
      component: string;
      metric: string;
      impact: 'high' | 'medium' | 'low';
      recommendation: string;
    }[];
    trends: {
      metric: string;
      direction: 'improving' | 'degrading' | 'stable';
      confidence: number;
    }[];
    recommendations: string[];
  }> {
    const response = await this.httpClient.get('/api/monitoring/performance-report', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取性能报告失败');
  }

  /**
   * 设置监控通知配置
   */
  public async setNotificationConfig(config: {
    email?: {
      enabled: boolean;
      recipients: string[];
      severityThreshold: 'info' | 'warning' | 'error' | 'critical';
    };
    webhook?: {
      enabled: boolean;
      url: string;
      secret?: string;
      severityThreshold: 'info' | 'warning' | 'error' | 'critical';
    };
    slack?: {
      enabled: boolean;
      webhookUrl: string;
      channel: string;
      severityThreshold: 'info' | 'warning' | 'error' | 'critical';
    };
  }): Promise<void> {
    const response = await this.httpClient.put('/api/monitoring/notification-config', config);
    
    if (!response.success) {
      throw new Error(response.message || '设置通知配置失败');
    }
  }

  /**
   * 获取监控通知配置
   */
  public async getNotificationConfig(): Promise<{
    email?: {
      enabled: boolean;
      recipients: string[];
      severityThreshold: string;
    };
    webhook?: {
      enabled: boolean;
      url: string;
      severityThreshold: string;
    };
    slack?: {
      enabled: boolean;
      webhookUrl: string;
      channel: string;
      severityThreshold: string;
    };
  }> {
    const response = await this.httpClient.get('/api/monitoring/notification-config');
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取通知配置失败');
  }

  /**
   * 测试通知配置
   */
  public async testNotification(type: 'email' | 'webhook' | 'slack'): Promise<{
    success: boolean;
    message: string;
    responseTime: number;
  }> {
    const response = await this.httpClient.post(`/api/monitoring/test-notification/${type}`);
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '测试通知失败');
  }

  /**
   * 获取监控统计
   */
  public async getMonitoringStats(
    timeRange?: { startTime: string; endTime: string }
  ): Promise<{
    period: string;
    totalAlerts: number;
    alertsByType: Record<string, number>;
    alertsByComponent: Record<string, number>;
    mttr: number; // Mean Time To Resolution
    mtbf: number; // Mean Time Between Failures
    availability: number;
    incidentCount: number;
  }> {
    const response = await this.httpClient.get('/api/monitoring/stats', {
      params: timeRange,
    });
    
    if (response.success && response.data) {
      return response.data;
    }
    
    throw new Error(response.message || '获取监控统计失败');
  }
}