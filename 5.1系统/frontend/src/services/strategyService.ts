import { apiCall, HttpMethod } from '../api/apiClient';

// 策略服务相关类型定义
export interface Strategy {
  id: string;
  name: string;
  type: 'triangular' | 'arbitrage' | 'grid' | 'market_making';
  status: 'running' | 'stopped' | 'paused' | 'error';
  config: any;
  performance: {
    profit: number;
    trades: number;
    success_rate: number;
  };
  created_at: string;
  updated_at: string;
}

export interface StrategyMetrics {
  cpu_usage: number;
  memory_usage: number;
  network_usage: number;
  trades_per_second: number;
  latency: number;
  error_rate: number;
}

export interface DebugSession {
  id: string;
  strategy_id: string;
  status: 'active' | 'inactive';
  breakpoints: Breakpoint[];
  variables: Record<string, any>;
  stack: any[];
  created_at: string;
}

export interface Breakpoint {
  id: string;
  line: number;
  condition?: string;
  enabled: boolean;
}

export interface HotReloadStatus {
  enabled: boolean;
  last_reload: string;
  status: 'success' | 'failed' | 'pending';
  changes: any[];
}

/**
 * 策略服务 - 38个API接口
 * 端口: 4003
 * 功能: 策略生命周期管理、实时监控、热重载
 */
export class StrategyService {
  
  // ==================== 策略生命周期管理API (12个) ====================
  
  /**
   * 列出所有策略
   */
  async listStrategies(): Promise<Strategy[]> {
    return apiCall(HttpMethod.GET, '/strategies/list');
  }
  
  /**
   * 获取策略详情
   */
  async getStrategy(id: string): Promise<Strategy> {
    return apiCall(HttpMethod.GET, `/strategies/${id}`);
  }
  
  /**
   * 启动策略
   */
  async startStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/start`);
  }
  
  /**
   * 停止策略
   */
  async stopStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/stop`);
  }
  
  /**
   * 重启策略
   */
  async restartStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/restart`);
  }
  
  /**
   * 暂停策略
   */
  async pauseStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/pause`);
  }
  
  /**
   * 恢复策略
   */
  async resumeStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/resume`);
  }
  
  /**
   * 获取策略状态
   */
  async getStrategyStatus(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/status`);
  }
  
  /**
   * 获取策略配置
   */
  async getStrategyConfig(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/config`);
  }
  
  /**
   * 更新策略配置
   */
  async updateStrategyConfig(id: string, config: any): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/config`, config);
  }
  
  /**
   * 获取策略日志
   */
  async getStrategyLogs(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/logs`);
  }
  
  /**
   * 获取策略指标
   */
  async getStrategyMetrics(id: string): Promise<StrategyMetrics> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/metrics`);
  }
  
  // ==================== 实时监控API (8个) ====================
  
  /**
   * 获取实时状态
   */
  async getRealtimeStatus(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/realtime');
  }
  
  /**
   * 获取系统健康状态
   */
  async getSystemHealth(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/health');
  }
  
  /**
   * 获取性能概览
   */
  async getPerformanceOverview(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/performance');
  }
  
  /**
   * 获取活跃告警
   */
  async getActiveAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/monitoring/alerts');
  }
  
  /**
   * 获取CPU指标
   */
  async getCpuMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/cpu');
  }
  
  /**
   * 获取内存指标
   */
  async getMemoryMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/memory');
  }
  
  /**
   * 获取网络指标
   */
  async getNetworkMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/network');
  }
  
  /**
   * 获取历史指标
   */
  async getHistoricalMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/history');
  }
  
  // ==================== 调试工具API (9个) ====================
  
  /**
   * 列出调试会话
   */
  async listDebugSessions(): Promise<DebugSession[]> {
    return apiCall(HttpMethod.GET, '/debug/sessions');
  }
  
  /**
   * 创建调试会话
   */
  async createDebugSession(strategy_id: string): Promise<DebugSession> {
    return apiCall(HttpMethod.POST, '/debug/sessions', { strategy_id });
  }
  
  /**
   * 获取调试会话
   */
  async getDebugSession(id: string): Promise<DebugSession> {
    return apiCall(HttpMethod.GET, `/debug/sessions/${id}`);
  }
  
  /**
   * 删除调试会话
   */
  async deleteDebugSession(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/debug/sessions/${id}`);
  }
  
  /**
   * 列出断点
   */
  async listBreakpoints(strategy_id: string): Promise<Breakpoint[]> {
    return apiCall(HttpMethod.GET, `/debug/breakpoints/${strategy_id}`);
  }
  
  /**
   * 添加断点
   */
  async addBreakpoint(strategy_id: string, line: number, condition?: string): Promise<Breakpoint> {
    return apiCall(HttpMethod.POST, `/debug/breakpoints/${strategy_id}`, { line, condition });
  }
  
  /**
   * 删除断点
   */
  async deleteBreakpoint(strategy_id: string, bp_id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/debug/breakpoints/${strategy_id}/${bp_id}`);
  }
  
  /**
   * 获取变量
   */
  async getVariables(strategy_id: string): Promise<Record<string, any>> {
    return apiCall(HttpMethod.GET, `/debug/variables/${strategy_id}`);
  }
  
  /**
   * 获取调用栈
   */
  async getCallStack(strategy_id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/debug/stack/${strategy_id}`);
  }
  
  // ==================== 热重载API (9个) ====================
  
  /**
   * 获取重载状态
   */
  async getHotReloadStatus(): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, '/hotreload/status');
  }
  
  /**
   * 获取策略重载状态
   */
  async getStrategyHotReloadStatus(strategy_id: string): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, `/hotreload/${strategy_id}/status`);
  }
  
  /**
   * 重载策略
   */
  async reloadStrategy(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/reload`);
  }
  
  /**
   * 启用热重载
   */
  async enableHotReload(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/enable`);
  }
  
  /**
   * 禁用热重载
   */
  async disableHotReload(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/disable`);
  }
  
  /**
   * 验证变更
   */
  async validateChanges(strategy_id: string, code: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/validate`, { code });
  }
  
  /**
   * 回滚变更
   */
  async rollbackChanges(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/rollback`);
  }
  
  /**
   * 获取重载历史
   */
  async getHotReloadHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/hotreload/history');
  }
  
  /**
   * 获取重载配置
   */
  async getHotReloadConfig(): Promise<any> {
    return apiCall(HttpMethod.GET, '/hotreload/config');
  }
}

// 导出单例实例
export const strategyService = new StrategyService(); 

// 策略服务相关类型定义
export interface Strategy {
  id: string;
  name: string;
  type: 'triangular' | 'arbitrage' | 'grid' | 'market_making';
  status: 'running' | 'stopped' | 'paused' | 'error';
  config: any;
  performance: {
    profit: number;
    trades: number;
    success_rate: number;
  };
  created_at: string;
  updated_at: string;
}

export interface StrategyMetrics {
  cpu_usage: number;
  memory_usage: number;
  network_usage: number;
  trades_per_second: number;
  latency: number;
  error_rate: number;
}

export interface DebugSession {
  id: string;
  strategy_id: string;
  status: 'active' | 'inactive';
  breakpoints: Breakpoint[];
  variables: Record<string, any>;
  stack: any[];
  created_at: string;
}

export interface Breakpoint {
  id: string;
  line: number;
  condition?: string;
  enabled: boolean;
}

export interface HotReloadStatus {
  enabled: boolean;
  last_reload: string;
  status: 'success' | 'failed' | 'pending';
  changes: any[];
}

/**
 * 策略服务 - 38个API接口
 * 端口: 4003
 * 功能: 策略生命周期管理、实时监控、热重载
 */
export class StrategyService {
  
  // ==================== 策略生命周期管理API (12个) ====================
  
  /**
   * 列出所有策略
   */
  async listStrategies(): Promise<Strategy[]> {
    return apiCall(HttpMethod.GET, '/strategies/list');
  }
  
  /**
   * 获取策略详情
   */
  async getStrategy(id: string): Promise<Strategy> {
    return apiCall(HttpMethod.GET, `/strategies/${id}`);
  }
  
  /**
   * 启动策略
   */
  async startStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/start`);
  }
  
  /**
   * 停止策略
   */
  async stopStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/stop`);
  }
  
  /**
   * 重启策略
   */
  async restartStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/restart`);
  }
  
  /**
   * 暂停策略
   */
  async pauseStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/pause`);
  }
  
  /**
   * 恢复策略
   */
  async resumeStrategy(id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/resume`);
  }
  
  /**
   * 获取策略状态
   */
  async getStrategyStatus(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/status`);
  }
  
  /**
   * 获取策略配置
   */
  async getStrategyConfig(id: string): Promise<any> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/config`);
  }
  
  /**
   * 更新策略配置
   */
  async updateStrategyConfig(id: string, config: any): Promise<void> {
    return apiCall(HttpMethod.POST, `/strategies/${id}/config`, config);
  }
  
  /**
   * 获取策略日志
   */
  async getStrategyLogs(id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/logs`);
  }
  
  /**
   * 获取策略指标
   */
  async getStrategyMetrics(id: string): Promise<StrategyMetrics> {
    return apiCall(HttpMethod.GET, `/strategies/${id}/metrics`);
  }
  
  // ==================== 实时监控API (8个) ====================
  
  /**
   * 获取实时状态
   */
  async getRealtimeStatus(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/realtime');
  }
  
  /**
   * 获取系统健康状态
   */
  async getSystemHealth(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/health');
  }
  
  /**
   * 获取性能概览
   */
  async getPerformanceOverview(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/performance');
  }
  
  /**
   * 获取活跃告警
   */
  async getActiveAlerts(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/monitoring/alerts');
  }
  
  /**
   * 获取CPU指标
   */
  async getCpuMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/cpu');
  }
  
  /**
   * 获取内存指标
   */
  async getMemoryMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/memory');
  }
  
  /**
   * 获取网络指标
   */
  async getNetworkMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/network');
  }
  
  /**
   * 获取历史指标
   */
  async getHistoricalMetrics(): Promise<any> {
    return apiCall(HttpMethod.GET, '/monitoring/metrics/history');
  }
  
  // ==================== 调试工具API (9个) ====================
  
  /**
   * 列出调试会话
   */
  async listDebugSessions(): Promise<DebugSession[]> {
    return apiCall(HttpMethod.GET, '/debug/sessions');
  }
  
  /**
   * 创建调试会话
   */
  async createDebugSession(strategy_id: string): Promise<DebugSession> {
    return apiCall(HttpMethod.POST, '/debug/sessions', { strategy_id });
  }
  
  /**
   * 获取调试会话
   */
  async getDebugSession(id: string): Promise<DebugSession> {
    return apiCall(HttpMethod.GET, `/debug/sessions/${id}`);
  }
  
  /**
   * 删除调试会话
   */
  async deleteDebugSession(id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/debug/sessions/${id}`);
  }
  
  /**
   * 列出断点
   */
  async listBreakpoints(strategy_id: string): Promise<Breakpoint[]> {
    return apiCall(HttpMethod.GET, `/debug/breakpoints/${strategy_id}`);
  }
  
  /**
   * 添加断点
   */
  async addBreakpoint(strategy_id: string, line: number, condition?: string): Promise<Breakpoint> {
    return apiCall(HttpMethod.POST, `/debug/breakpoints/${strategy_id}`, { line, condition });
  }
  
  /**
   * 删除断点
   */
  async deleteBreakpoint(strategy_id: string, bp_id: string): Promise<void> {
    return apiCall(HttpMethod.DELETE, `/debug/breakpoints/${strategy_id}/${bp_id}`);
  }
  
  /**
   * 获取变量
   */
  async getVariables(strategy_id: string): Promise<Record<string, any>> {
    return apiCall(HttpMethod.GET, `/debug/variables/${strategy_id}`);
  }
  
  /**
   * 获取调用栈
   */
  async getCallStack(strategy_id: string): Promise<any[]> {
    return apiCall(HttpMethod.GET, `/debug/stack/${strategy_id}`);
  }
  
  // ==================== 热重载API (9个) ====================
  
  /**
   * 获取重载状态
   */
  async getHotReloadStatus(): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, '/hotreload/status');
  }
  
  /**
   * 获取策略重载状态
   */
  async getStrategyHotReloadStatus(strategy_id: string): Promise<HotReloadStatus> {
    return apiCall(HttpMethod.GET, `/hotreload/${strategy_id}/status`);
  }
  
  /**
   * 重载策略
   */
  async reloadStrategy(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/reload`);
  }
  
  /**
   * 启用热重载
   */
  async enableHotReload(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/enable`);
  }
  
  /**
   * 禁用热重载
   */
  async disableHotReload(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/disable`);
  }
  
  /**
   * 验证变更
   */
  async validateChanges(strategy_id: string, code: string): Promise<any> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/validate`, { code });
  }
  
  /**
   * 回滚变更
   */
  async rollbackChanges(strategy_id: string): Promise<void> {
    return apiCall(HttpMethod.POST, `/hotreload/${strategy_id}/rollback`);
  }
  
  /**
   * 获取重载历史
   */
  async getHotReloadHistory(): Promise<any[]> {
    return apiCall(HttpMethod.GET, '/hotreload/history');
  }
  
  /**
   * 获取重载配置
   */
  async getHotReloadConfig(): Promise<any> {
    return apiCall(HttpMethod.GET, '/hotreload/config');
  }
}

// 导出单例实例
export const strategyService = new StrategyService(); 