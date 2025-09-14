import { apiCall, HttpMethod, wsManager } from '../api/apiClient';

// 日志相关类型定义
export interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  service: string;
  module: string;
  message: string;
  metadata?: any;
}

export interface LogStreamQuery {
  service?: string;
  level?: string;
  module?: string;
  query?: string;
  lines?: number;
  hours?: number;
}

export interface LogConfig {
  levels: Record<string, string>;
  filters: any[];
  retention: { days: number };
  rotation: { size: string };
  storage: { path: string };
  format: string;
  sampling: { rate: number };
}

export interface LogAnalysis {
  stats: any;
  trends: any;
  anomalies: any[];
  patterns: any[];
  errors: any[];
  performance: any;
  frequency: any;
  correlations: any[];
  insights: any[];
}

/**
 * 日志服务 - 45个API接口
 * 端口: 4001
 * 功能: 系统日志收集、分析、实时流处理
 */
export class LoggingService {
  
  // ==================== 实时日志流API (15个) ====================
  
  /**
   * 获取实时日志流
   */
  async getRealtimeLogStream(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/realtime');
  }
  
  /**
   * 按服务过滤日志
   */
  async getLogsByService(service: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-service/${service}`);
  }
  
  /**
   * 按级别过滤日志
   */
  async getLogsByLevel(level: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-level/${level}`);
  }
  
  /**
   * 按模块过滤日志
   */
  async getLogsByModule(module: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-module/${module}`);
  }
  
  /**
   * 搜索日志内容
   */
  async searchLogs(query: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.POST, '/logs/stream/search', { query });
  }
  
  /**
   * 尾随日志输出
   */
  async tailLogs(lines: number = 100): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/tail?lines=${lines}`);
  }
  
  /**
   * 跟踪日志变化
   */
  async followLogs(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/follow');
  }
  
  /**
   * 获取缓冲区日志
   */
  async getBufferLogs(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/buffer');
  }
  
  /**
   * 获取历史日志
   */
  async getHistoryLogs(hours: number = 24): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/history?hours=${hours}`);
  }
  
  /**
   * 导出日志数据
   */
  async exportLogs(format: string = 'json'): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/stream/export', { format });
  }
  
  /**
   * 获取流处理统计
   */
  async getStreamStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/stream/stats');
  }
  
  /**
   * 暂停日志流
   */
  async pauseLogStream(): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/stream/pause');
  }
  
  /**
   * 恢复日志流
   */
  async resumeLogStream(): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/stream/resume');
  }
  
  /**
   * WebSocket实时日志连接
   */
  connectRealtimeLogs(onMessage: (data: LogEntry) => void, onError?: (error: any) => void): WebSocket {
    return wsManager.connect('/logs/realtime', onMessage, onError);
  }
  
  /**
   * WebSocket过滤日志连接
   */
  connectFilteredLogs(onMessage: (data: LogEntry) => void, onError?: (error: any) => void): WebSocket {
    return wsManager.connect('/logs/filtered', onMessage, onError);
  }
  
  // ==================== 日志配置API (18个) ====================
  
  /**
   * 获取日志级别配置
   */
  async getLogLevels(): Promise<Record<string, string>> {
    return apiCall(HttpMethod.GET, '/logs/config/levels');
  }
  
  /**
   * 设置日志级别配置
   */
  async setLogLevels(level: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/levels', { level });
  }
  
  /**
   * 获取服务日志级别
   */
  async getServiceLogLevel(service: string): Promise<string> {
    return apiCall(HttpMethod.GET, `/logs/config/levels/${service}`);
  }
  
  /**
   * 设置服务日志级别
   */
  async setServiceLogLevel(service: string, level: string): Promise<void> {
    return apiCall(HttpMethod.PUT, `/logs/config/levels/${service}`, { level });
  }
  
  /**
   * 获取日志过滤器
   */
  async getLogFilters(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/config/filters');
  }
  
  /**
   * 添加日志过滤器
   */
  async addLogFilter(pattern: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/config/filters', { pattern });
  }
  
  /**
   * 删除日志过滤器
   */
  async deleteLogFilter(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/logs/config/filters/${id}`);
  }
  
  /**
   * 获取保留策略
   */
  async getRetentionPolicy(): Promise<{ days: number }> {
    return apiCall(HttpMethod.GET, '/logs/config/retention');
  }
  
  /**
   * 设置保留策略
   */
  async setRetentionPolicy(days: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/retention', { days });
  }
  
  /**
   * 获取轮转配置
   */
  async getRotationConfig(): Promise<{ size: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/rotation');
  }
  
  /**
   * 设置轮转配置
   */
  async setRotationConfig(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/rotation', { size });
  }
  
  /**
   * 获取存储配置
   */
  async getStorageConfig(): Promise<{ path: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/storage');
  }
  
  /**
   * 设置存储配置
   */
  async setStorageConfig(path: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/storage', { path });
  }
  
  /**
   * 获取日志格式
   */
  async getLogFormat(): Promise<{ format: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/format');
  }
  
  /**
   * 设置日志格式
   */
  async setLogFormat(format: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/format', { format });
  }
  
  /**
   * 获取采样配置
   */
  async getSamplingConfig(): Promise<{ rate: number }> {
    return apiCall(HttpMethod.GET, '/logs/config/sampling');
  }
  
  /**
   * 设置采样配置
   */
  async setSamplingConfig(rate: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/sampling', { rate });
  }
  
  /**
   * 导出配置
   */
  async exportConfig(): Promise<LogConfig> {
    return apiCall(HttpMethod.POST, '/logs/config/export');
  }
  
  // ==================== 日志分析API (12个) ====================
  
  /**
   * 获取日志统计
   */
  async getLogStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/stats');
  }
  
  /**
   * 获取日志趋势
   */
  async getLogTrends(period: string = '24h'): Promise<any> {
    return apiCall(HttpMethod.GET, `/logs/analysis/trends?period=${period}`);
  }
  
  /**
   * 异常检测
   */
  async detectAnomalies(threshold: number = 0.95): Promise<any[]> {
    return apiCall(HttpMethod.POST, '/logs/analysis/anomaly', { threshold });
  }
  
  /**
   * 模式查找
   */
  async findPatterns(regex: string): Promise<any[]> {
    return apiCall(HttpMethod.POST, '/logs/analysis/patterns', { regex });
  }
  
  /**
   * 错误分析
   */
  async analyzeErrors(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/errors');
  }
  
  /**
   * 性能分析
   */
  async analyzePerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/performance');
  }
  
  /**
   * 频率分析
   */
  async analyzeFrequency(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/frequency');
  }
  
  /**
   * 关联分析
   */
  async analyzeCorrelations(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/correlations');
  }
  
  /**
   * 自定义分析
   */
  async customAnalysis(query: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/analysis/custom', { query });
  }
  
  /**
   * 获取分析报告
   */
  async getAnalysisReports(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/reports');
  }
  
  /**
   * 创建分析报告
   */
  async createAnalysisReport(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/analysis/reports', { name });
  }
  
  /**
   * 获取洞察
   */
  async getInsights(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/insights');
  }
}

// 导出单例实例
export const loggingService = new LoggingService(); 

// 日志相关类型定义
export interface LogEntry {
  id: string;
  timestamp: string;
  level: 'debug' | 'info' | 'warn' | 'error';
  service: string;
  module: string;
  message: string;
  metadata?: any;
}

export interface LogStreamQuery {
  service?: string;
  level?: string;
  module?: string;
  query?: string;
  lines?: number;
  hours?: number;
}

export interface LogConfig {
  levels: Record<string, string>;
  filters: any[];
  retention: { days: number };
  rotation: { size: string };
  storage: { path: string };
  format: string;
  sampling: { rate: number };
}

export interface LogAnalysis {
  stats: any;
  trends: any;
  anomalies: any[];
  patterns: any[];
  errors: any[];
  performance: any;
  frequency: any;
  correlations: any[];
  insights: any[];
}

/**
 * 日志服务 - 45个API接口
 * 端口: 4001
 * 功能: 系统日志收集、分析、实时流处理
 */
export class LoggingService {
  
  // ==================== 实时日志流API (15个) ====================
  
  /**
   * 获取实时日志流
   */
  async getRealtimeLogStream(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/realtime');
  }
  
  /**
   * 按服务过滤日志
   */
  async getLogsByService(service: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-service/${service}`);
  }
  
  /**
   * 按级别过滤日志
   */
  async getLogsByLevel(level: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-level/${level}`);
  }
  
  /**
   * 按模块过滤日志
   */
  async getLogsByModule(module: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/by-module/${module}`);
  }
  
  /**
   * 搜索日志内容
   */
  async searchLogs(query: string): Promise<LogEntry[]> {
    return apiCall(HttpMethod.POST, '/logs/stream/search', { query });
  }
  
  /**
   * 尾随日志输出
   */
  async tailLogs(lines: number = 100): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/tail?lines=${lines}`);
  }
  
  /**
   * 跟踪日志变化
   */
  async followLogs(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/follow');
  }
  
  /**
   * 获取缓冲区日志
   */
  async getBufferLogs(): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, '/logs/stream/buffer');
  }
  
  /**
   * 获取历史日志
   */
  async getHistoryLogs(hours: number = 24): Promise<LogEntry[]> {
    return apiCall(HttpMethod.GET, `/logs/stream/history?hours=${hours}`);
  }
  
  /**
   * 导出日志数据
   */
  async exportLogs(format: string = 'json'): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/stream/export', { format });
  }
  
  /**
   * 获取流处理统计
   */
  async getStreamStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/stream/stats');
  }
  
  /**
   * 暂停日志流
   */
  async pauseLogStream(): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/stream/pause');
  }
  
  /**
   * 恢复日志流
   */
  async resumeLogStream(): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/stream/resume');
  }
  
  /**
   * WebSocket实时日志连接
   */
  connectRealtimeLogs(onMessage: (data: LogEntry) => void, onError?: (error: any) => void): WebSocket {
    return wsManager.connect('/logs/realtime', onMessage, onError);
  }
  
  /**
   * WebSocket过滤日志连接
   */
  connectFilteredLogs(onMessage: (data: LogEntry) => void, onError?: (error: any) => void): WebSocket {
    return wsManager.connect('/logs/filtered', onMessage, onError);
  }
  
  // ==================== 日志配置API (18个) ====================
  
  /**
   * 获取日志级别配置
   */
  async getLogLevels(): Promise<Record<string, string>> {
    return apiCall(HttpMethod.GET, '/logs/config/levels');
  }
  
  /**
   * 设置日志级别配置
   */
  async setLogLevels(level: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/levels', { level });
  }
  
  /**
   * 获取服务日志级别
   */
  async getServiceLogLevel(service: string): Promise<string> {
    return apiCall(HttpMethod.GET, `/logs/config/levels/${service}`);
  }
  
  /**
   * 设置服务日志级别
   */
  async setServiceLogLevel(service: string, level: string): Promise<void> {
    return apiCall(HttpMethod.PUT, `/logs/config/levels/${service}`, { level });
  }
  
  /**
   * 获取日志过滤器
   */
  async getLogFilters(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/config/filters');
  }
  
  /**
   * 添加日志过滤器
   */
  async addLogFilter(pattern: string): Promise<void> {
    return apiCall(HttpMethod.POST, '/logs/config/filters', { pattern });
  }
  
  /**
   * 删除日志过滤器
   */
  async deleteLogFilter(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/logs/config/filters/${id}`);
  }
  
  /**
   * 获取保留策略
   */
  async getRetentionPolicy(): Promise<{ days: number }> {
    return apiCall(HttpMethod.GET, '/logs/config/retention');
  }
  
  /**
   * 设置保留策略
   */
  async setRetentionPolicy(days: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/retention', { days });
  }
  
  /**
   * 获取轮转配置
   */
  async getRotationConfig(): Promise<{ size: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/rotation');
  }
  
  /**
   * 设置轮转配置
   */
  async setRotationConfig(size: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/rotation', { size });
  }
  
  /**
   * 获取存储配置
   */
  async getStorageConfig(): Promise<{ path: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/storage');
  }
  
  /**
   * 设置存储配置
   */
  async setStorageConfig(path: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/storage', { path });
  }
  
  /**
   * 获取日志格式
   */
  async getLogFormat(): Promise<{ format: string }> {
    return apiCall(HttpMethod.GET, '/logs/config/format');
  }
  
  /**
   * 设置日志格式
   */
  async setLogFormat(format: string): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/format', { format });
  }
  
  /**
   * 获取采样配置
   */
  async getSamplingConfig(): Promise<{ rate: number }> {
    return apiCall(HttpMethod.GET, '/logs/config/sampling');
  }
  
  /**
   * 设置采样配置
   */
  async setSamplingConfig(rate: number): Promise<void> {
    return apiCall(HttpMethod.PUT, '/logs/config/sampling', { rate });
  }
  
  /**
   * 导出配置
   */
  async exportConfig(): Promise<LogConfig> {
    return apiCall(HttpMethod.POST, '/logs/config/export');
  }
  
  // ==================== 日志分析API (12个) ====================
  
  /**
   * 获取日志统计
   */
  async getLogStats(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/stats');
  }
  
  /**
   * 获取日志趋势
   */
  async getLogTrends(period: string = '24h'): Promise<any> {
    return apiCall(HttpMethod.GET, `/logs/analysis/trends?period=${period}`);
  }
  
  /**
   * 异常检测
   */
  async detectAnomalies(threshold: number = 0.95): Promise<any[]> {
    return apiCall(HttpMethod.POST, '/logs/analysis/anomaly', { threshold });
  }
  
  /**
   * 模式查找
   */
  async findPatterns(regex: string): Promise<any[]> {
    return apiCall(HttpMethod.POST, '/logs/analysis/patterns', { regex });
  }
  
  /**
   * 错误分析
   */
  async analyzeErrors(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/errors');
  }
  
  /**
   * 性能分析
   */
  async analyzePerformance(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/performance');
  }
  
  /**
   * 频率分析
   */
  async analyzeFrequency(): Promise<any> {
    return apiCall(HttpMethod.GET, '/logs/analysis/frequency');
  }
  
  /**
   * 关联分析
   */
  async analyzeCorrelations(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/correlations');
  }
  
  /**
   * 自定义分析
   */
  async customAnalysis(query: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/analysis/custom', { query });
  }
  
  /**
   * 获取分析报告
   */
  async getAnalysisReports(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/reports');
  }
  
  /**
   * 创建分析报告
   */
  async createAnalysisReport(name: string): Promise<any> {
    return apiCall(HttpMethod.POST, '/logs/analysis/reports', { name });
  }
  
  /**
   * 获取洞察
   */
  async getInsights(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/logs/analysis/insights');
  }
}

// 导出单例实例
export const loggingService = new LoggingService(); 